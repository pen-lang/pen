use super::error::CompileError;
use crate::{
    closure, context::Context, expression, reference_count, type_, yield_::yield_function_type,
};
use std::sync::LazyLock;

const CLOSURE_NAME: &str = "_closure";

static ENTRY_FUNCTION_DEFINITION_OPTIONS: LazyLock<fmm::ir::FunctionDefinitionOptions> =
    LazyLock::new(|| {
        fmm::ir::FunctionDefinitionOptions::new()
            .set_address_named(false)
            .set_calling_convention(fmm::types::CallingConvention::Source)
            .set_linkage(fmm::ir::Linkage::Internal)
    });

pub fn compile(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
    global: bool,
    variables: &plist::FlailMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(if definition.is_thunk() {
        compile_thunk(context, definition, global, variables)?
    } else {
        compile_non_thunk(context, definition, global, variables)?
    })
}

fn compile_non_thunk(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
    global: bool,
    variables: &plist::FlailMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    context.module_builder().define_anonymous_function(
        compile_arguments(context, definition),
        type_::compile(context, definition.result_type()),
        |builder| {
            Ok(builder.return_(compile_body(
                context, &builder, definition, global, variables,
            )?))
        },
        ENTRY_FUNCTION_DEFINITION_OPTIONS.clone(),
    )
}

// Entry functions of thunks need to be loaded atomically to make thunk update
// thread-safe.
//
// A relaxed ordering is allowed to load any of those entry functions since they
// should guarantee memory operation ordering by themselves.
fn compile_thunk(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
    global: bool,
    variables: &plist::FlailMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    compile_initial_thunk_entry(
        context,
        definition,
        global,
        compile_normal_thunk_entry(context, definition)?,
        compile_locked_thunk_entry(context, definition)?,
        variables,
    )
}

fn compile_body(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    definition: &mir::ir::FunctionDefinition,
    global: bool,
    variables: &plist::FlailMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let environment_pointer = compile_environment_pointer(context, definition)?;

    expression::compile(
        context,
        builder,
        definition.body(),
        &variables
            .insert_iter(
                definition
                    .environment()
                    .iter()
                    .enumerate()
                    .map(|(index, free_variable)| -> Result<_, CompileError> {
                        Ok((
                            free_variable.name().into(),
                            reference_count::clone(
                                context,
                                builder,
                                &builder.load(fmm::build::record_address(
                                    environment_pointer.clone(),
                                    index,
                                )?)?,
                                free_variable.type_(),
                            )?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .insert_iter(if global {
                None
            } else {
                Some((
                    definition.name().into(),
                    compile_closure_pointer(context, definition.type_())?,
                ))
            })
            .insert_iter(definition.arguments().iter().map(|argument| {
                (
                    argument.name().into(),
                    fmm::build::variable(
                        argument.name(),
                        type_::compile(context, argument.type_()),
                    ),
                )
            })),
    )
}

fn compile_initial_thunk_entry(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
    global: bool,
    normal_entry_function: fmm::build::TypedExpression,
    lock_entry_function: fmm::build::TypedExpression,
    variables: &plist::FlailMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let entry_function_name = context.module_builder().generate_name();
    let arguments = compile_arguments(context, definition);

    context.module_builder().define_function(
        &entry_function_name,
        arguments.clone(),
        type_::compile(context, definition.result_type()),
        |builder| {
            let closure_pointer = compile_closure_pointer(context, definition.type_())?;
            let entry_function_pointer =
                closure::get_entry_function_pointer(closure_pointer.clone())?;
            let synchronized =
                reference_count::pointer::is_synchronized(&builder, &closure_pointer)?;

            builder.if_(
                synchronized.clone(),
                |builder| -> Result<_, CompileError> {
                    builder.if_(
                        builder.compare_and_swap(
                            entry_function_pointer.clone(),
                            fmm::build::variable(
                                &entry_function_name,
                                type_::compile_entry_function(context, definition.type_()),
                            ),
                            lock_entry_function.clone(),
                            fmm::ir::AtomicOrdering::Acquire,
                            fmm::ir::AtomicOrdering::Relaxed,
                        ),
                        |builder| -> Result<_, CompileError> {
                            Ok(builder.branch(fmm::ir::void_value()))
                        },
                        |builder| {
                            // TODO Use an entry function loaded by a CAS instruction above.
                            Ok(builder.return_(builder.call(
                                builder.atomic_load(
                                    entry_function_pointer.clone(),
                                    fmm::ir::AtomicOrdering::Relaxed,
                                )?,
                                compile_argument_variables(&arguments),
                            )?))
                        },
                    )?;

                    Ok(builder.branch(fmm::ir::void_value()))
                },
                |builder| Ok(builder.branch(fmm::ir::void_value())),
            )?;

            let closure_pointer = reference_count::clone(
                context,
                &builder,
                &closure_pointer,
                &definition.type_().clone().into(),
            )?;

            let value = compile_body(context, &builder, definition, global, variables)?;

            let environment_pointer = compile_environment_pointer(context, definition)?;

            // TODO Remove these extra drops of free variables when we move them into
            // function bodies rather than cloning them.
            // See also https://github.com/pen-lang/pen/issues/295.
            for (index, free_variable) in definition.environment().iter().enumerate() {
                reference_count::drop(
                    context,
                    &builder,
                    &builder.load(fmm::build::record_address(
                        environment_pointer.clone(),
                        index,
                    )?)?,
                    free_variable.type_(),
                )?;
            }

            builder.store(
                reference_count::clone(context, &builder, &value, definition.result_type())?,
                compile_thunk_value_pointer(context, definition)?,
            );

            builder.store(
                closure::metadata::compile_normal_thunk(context, definition)?,
                closure::get_metadata_pointer(closure_pointer.clone())?,
            );

            builder.if_(
                synchronized,
                |builder| -> Result<_, CompileError> {
                    builder.atomic_store(
                        normal_entry_function.clone(),
                        entry_function_pointer.clone(),
                        fmm::ir::AtomicOrdering::Release,
                    );

                    Ok(builder.branch(fmm::ir::void_value()))
                },
                |builder| {
                    builder.store(
                        normal_entry_function.clone(),
                        entry_function_pointer.clone(),
                    );

                    Ok(builder.branch(fmm::ir::void_value()))
                },
            )?;

            reference_count::drop(
                context,
                &builder,
                &closure_pointer,
                &definition.type_().clone().into(),
            )?;

            Ok(builder.return_(value))
        },
        ENTRY_FUNCTION_DEFINITION_OPTIONS.clone(),
    )
}

fn compile_normal_thunk_entry(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
) -> Result<fmm::build::TypedExpression, CompileError> {
    context.module_builder().define_anonymous_function(
        compile_arguments(context, definition),
        type_::compile(context, definition.result_type()),
        |builder| {
            let closure_pointer = compile_closure_pointer(context, definition.type_())?;

            builder.if_(
                reference_count::pointer::is_synchronized(&builder, &closure_pointer)?,
                |builder| -> Result<_, CompileError> {
                    builder.atomic_load(
                        closure::get_entry_function_pointer(closure_pointer.clone())?,
                        fmm::ir::AtomicOrdering::Acquire,
                    )?;

                    Ok(builder.branch(fmm::ir::void_value()))
                },
                |builder| Ok(builder.branch(fmm::ir::void_value())),
            )?;

            let value = reference_count::clone(
                context,
                &builder,
                &builder.load(compile_thunk_value_pointer(context, definition)?)?,
                definition.result_type(),
            )?;

            reference_count::drop(
                context,
                &builder,
                &closure_pointer,
                &definition.type_().clone().into(),
            )?;

            Ok(builder.return_(value))
        },
        ENTRY_FUNCTION_DEFINITION_OPTIONS.clone(),
    )
}

fn compile_locked_thunk_entry(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let arguments = compile_arguments(context, definition);

    context.module_builder().define_function(
        context.module_builder().generate_name(),
        arguments.clone(),
        type_::compile(context, definition.result_type()),
        |builder| {
            builder.call(
                fmm::build::variable(
                    &context.configuration().yield_function_name,
                    yield_function_type(),
                ),
                vec![],
            )?;

            Ok(builder.return_(builder.call(
                builder.atomic_load(
                    closure::get_entry_function_pointer(compile_closure_pointer(
                        context,
                        definition.type_(),
                    )?)?,
                    fmm::ir::AtomicOrdering::Relaxed,
                )?,
                compile_argument_variables(&arguments),
            )?))
        },
        ENTRY_FUNCTION_DEFINITION_OPTIONS.clone(),
    )
}

fn compile_arguments(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
) -> Vec<fmm::ir::Argument> {
    [fmm::ir::Argument::new(
        CLOSURE_NAME,
        type_::compile_untyped_closure_pointer(),
    )]
    .into_iter()
    .chain(definition.arguments().iter().map(|argument| {
        fmm::ir::Argument::new(argument.name(), type_::compile(context, argument.type_()))
    }))
    .collect()
}

fn compile_argument_variables(arguments: &[fmm::ir::Argument]) -> Vec<fmm::build::TypedExpression> {
    arguments
        .iter()
        .map(|argument| fmm::build::variable(argument.name(), argument.type_().clone()))
        .collect()
}

fn compile_thunk_value_pointer(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::union_address(compile_payload_pointer(context, definition)?, 1)?.into())
}

fn compile_environment_pointer(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let payload_pointer = compile_payload_pointer(context, definition)?;

    Ok(if definition.is_thunk() {
        fmm::build::union_address(payload_pointer, 0)?.into()
    } else {
        payload_pointer
    })
}

fn compile_payload_pointer(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
) -> Result<fmm::build::TypedExpression, CompileError> {
    closure::get_payload_pointer(fmm::build::bit_cast(
        fmm::types::Pointer::new(type_::compile_sized_closure(context, definition)),
        compile_untyped_closure_pointer(),
    ))
}

fn compile_closure_pointer(
    context: &Context,
    function_type: &mir::types::Function,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    Ok(fmm::build::bit_cast(
        fmm::types::Pointer::new(type_::compile_unsized_closure(context, function_type)),
        compile_untyped_closure_pointer(),
    )
    .into())
}

fn compile_untyped_closure_pointer() -> fmm::build::TypedExpression {
    fmm::build::variable(CLOSURE_NAME, type_::compile_untyped_closure_pointer())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::CONFIGURATION;
    use mir::test::ModuleFake;

    #[test]
    fn do_not_overwrite_global_functions_in_variables() {
        let function_type = mir::types::Function::new(vec![], mir::types::Type::Number);
        let context = Context::new(&mir::ir::Module::empty(), CONFIGURATION.clone());

        compile(
            &context,
            &mir::ir::FunctionDefinition::new(
                "f",
                vec![],
                mir::types::Type::Number,
                mir::ir::LetRecursive::new(
                    mir::ir::FunctionDefinition::new(
                        "g",
                        vec![],
                        mir::types::Type::Number,
                        mir::ir::Call::new(
                            function_type.clone(),
                            mir::ir::Variable::new("f"),
                            vec![],
                        ),
                    ),
                    mir::ir::Call::new(function_type.clone(), mir::ir::Variable::new("g"), vec![]),
                ),
            ),
            true,
            &[(
                "f".into(),
                fmm::build::TypedExpression::new(
                    fmm::ir::Variable::new("f"),
                    fmm::types::Pointer::new(type_::compile_unsized_closure(
                        &context,
                        &function_type,
                    )),
                ),
            )]
            .into_iter()
            .collect(),
        )
        .unwrap();

        insta::assert_snapshot!(fmm::analysis::format::format_module(
            &context.into_module_builder().into_module()
        ));
    }
}
