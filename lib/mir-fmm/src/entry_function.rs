use super::error::CompileError;
use crate::{
    closure, context::Context, expression, pointer, reference_count, type_,
    yield_::YIELD_FUNCTION_TYPE,
};
use fnv::FnvHashMap;

const CLOSURE_NAME: &str = "_closure";

pub fn compile(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
    global: bool,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
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
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    context.module_builder().define_anonymous_function(
        compile_arguments(definition, context.types()),
        |instruction_builder| {
            Ok(instruction_builder.return_(compile_body(
                context,
                &instruction_builder,
                definition,
                global,
                variables,
            )?))
        },
        type_::compile(definition.result_type(), context.types()),
        fmm::types::CallingConvention::Source,
    )
}

fn compile_thunk(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
    global: bool,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
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
    instruction_builder: &fmm::build::InstructionBuilder,
    definition: &mir::ir::FunctionDefinition,
    global: bool,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let environment_pointer = compile_environment_pointer(definition, context.types())?;

    expression::compile(
        context,
        instruction_builder,
        definition.body(),
        &variables
            .clone()
            .into_iter()
            .chain(
                definition
                    .environment()
                    .iter()
                    .enumerate()
                    .map(|(index, free_variable)| -> Result<_, CompileError> {
                        Ok((
                            free_variable.name().into(),
                            reference_count::clone(
                                instruction_builder,
                                &instruction_builder.load(fmm::build::record_address(
                                    environment_pointer.clone(),
                                    index,
                                )?)?,
                                free_variable.type_(),
                                context.types(),
                            )?,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .chain(if global {
                vec![]
            } else {
                vec![(
                    definition.name().into(),
                    compile_closure_pointer(definition.type_(), context.types())?,
                )]
            })
            .chain(definition.arguments().iter().map(|argument| {
                (
                    argument.name().into(),
                    fmm::build::variable(
                        argument.name(),
                        type_::compile(argument.type_(), context.types()),
                    ),
                )
            }))
            .collect(),
    )
}

fn compile_initial_thunk_entry(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
    global: bool,
    normal_entry_function: fmm::build::TypedExpression,
    lock_entry_function: fmm::build::TypedExpression,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let entry_function_name = context.module_builder().generate_name();
    let arguments = compile_arguments(definition, context.types());

    context.module_builder().define_function(
        &entry_function_name,
        arguments.clone(),
        |instruction_builder| {
            let entry_function_pointer =
                compile_entry_function_pointer(definition, context.types())?;

            instruction_builder.if_(
                instruction_builder.compare_and_swap(
                    entry_function_pointer.clone(),
                    fmm::build::variable(
                        &entry_function_name,
                        type_::compile_entry_function(definition.type_(), context.types()),
                    ),
                    lock_entry_function.clone(),
                    fmm::ir::AtomicOrdering::Acquire,
                    fmm::ir::AtomicOrdering::Relaxed,
                ),
                |instruction_builder| -> Result<_, CompileError> {
                    let closure = reference_count::clone(
                        &instruction_builder,
                        &compile_closure_pointer(definition.type_(), context.types())?,
                        &definition.type_().clone().into(),
                        context.types(),
                    )?;

                    let value =
                        compile_body(context, &instruction_builder, definition, global, variables)?;

                    let environment_pointer =
                        compile_environment_pointer(definition, context.types())?;

                    // TODO Remove these extra drops of free variables when we move them in function
                    // bodies rather than cloning them.
                    // See also https://github.com/pen-lang/pen/issues/295.
                    for (index, free_variable) in definition.environment().iter().enumerate() {
                        reference_count::drop(
                            &instruction_builder,
                            &instruction_builder.load(fmm::build::record_address(
                                environment_pointer.clone(),
                                index,
                            )?)?,
                            free_variable.type_(),
                            context.types(),
                        )?;
                    }

                    instruction_builder.store(
                        reference_count::clone(
                            &instruction_builder,
                            &value,
                            definition.result_type(),
                            context.types(),
                        )?,
                        compile_thunk_value_pointer(definition, context.types())?,
                    );

                    instruction_builder.store(
                        closure::compile_normal_thunk_drop_function(context, definition)?,
                        compile_drop_function_pointer(definition, context.types())?,
                    );

                    instruction_builder.atomic_store(
                        normal_entry_function.clone(),
                        entry_function_pointer.clone(),
                        fmm::ir::AtomicOrdering::Release,
                    );

                    reference_count::drop(
                        &instruction_builder,
                        &closure,
                        &definition.type_().clone().into(),
                        context.types(),
                    )?;

                    Ok(instruction_builder.return_(value))
                },
                |instruction_builder| {
                    Ok(instruction_builder.return_(instruction_builder.call(
                        instruction_builder.atomic_load(
                            compile_entry_function_pointer(definition, context.types())?,
                            fmm::ir::AtomicOrdering::Acquire,
                        )?,
                        compile_argument_variables(&arguments),
                    )?))
                },
            )?;

            Ok(instruction_builder.unreachable())
        },
        type_::compile(definition.result_type(), context.types()),
        fmm::types::CallingConvention::Source,
        fmm::ir::Linkage::Internal,
    )
}

fn compile_normal_thunk_entry(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
) -> Result<fmm::build::TypedExpression, CompileError> {
    context.module_builder().define_anonymous_function(
        compile_arguments(definition, context.types()),
        |instruction_builder| {
            compile_normal_body(&instruction_builder, definition, context.types())
        },
        type_::compile(definition.result_type(), context.types()),
        fmm::types::CallingConvention::Source,
    )
}

fn compile_locked_thunk_entry(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let entry_function_name = context.module_builder().generate_name();
    let entry_function = fmm::build::variable(
        &entry_function_name,
        type_::compile_entry_function(definition.type_(), context.types()),
    );
    let arguments = compile_arguments(definition, context.types());

    context.module_builder().define_function(
        &entry_function_name,
        arguments.clone(),
        |instruction_builder| {
            instruction_builder.if_(
                pointer::equal(
                    instruction_builder.atomic_load(
                        compile_entry_function_pointer(definition, context.types())?,
                        fmm::ir::AtomicOrdering::Acquire,
                    )?,
                    entry_function.clone(),
                )?,
                |instruction_builder| {
                    instruction_builder.call(
                        fmm::build::variable(
                            &context.configuration().yield_function_name,
                            YIELD_FUNCTION_TYPE.clone(),
                        ),
                        vec![],
                    )?;

                    Ok(instruction_builder.return_(instruction_builder.call(
                        entry_function.clone(),
                        compile_argument_variables(&arguments),
                    )?))
                },
                |instruction_builder| {
                    compile_normal_body(&instruction_builder, definition, context.types())
                },
            )?;

            Ok(instruction_builder.unreachable())
        },
        type_::compile(definition.result_type(), context.types()),
        fmm::types::CallingConvention::Source,
        fmm::ir::Linkage::Internal,
    )
}

fn compile_normal_body(
    instruction_builder: &fmm::build::InstructionBuilder,
    definition: &mir::ir::FunctionDefinition,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::ir::Block, CompileError> {
    let value = reference_count::clone(
        instruction_builder,
        &instruction_builder.load(compile_thunk_value_pointer(definition, types)?)?,
        definition.result_type(),
        types,
    )?;

    reference_count::drop(
        instruction_builder,
        &compile_closure_pointer(definition.type_(), types)?,
        &definition.type_().clone().into(),
        types,
    )?;

    Ok(instruction_builder.return_(value))
}

fn compile_entry_function_pointer(
    definition: &mir::ir::FunctionDefinition,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    closure::compile_entry_function_pointer(compile_closure_pointer(definition.type_(), types)?)
}

fn compile_drop_function_pointer(
    definition: &mir::ir::FunctionDefinition,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    closure::compile_drop_function_pointer(compile_closure_pointer(definition.type_(), types)?)
}

fn compile_arguments(
    definition: &mir::ir::FunctionDefinition,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Vec<fmm::ir::Argument> {
    [fmm::ir::Argument::new(
        CLOSURE_NAME,
        type_::compile_untyped_closure_pointer(),
    )]
    .into_iter()
    .chain(definition.arguments().iter().map(|argument| {
        fmm::ir::Argument::new(argument.name(), type_::compile(argument.type_(), types))
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
    definition: &mir::ir::FunctionDefinition,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::union_address(compile_payload_pointer(definition, types)?, 1)?.into())
}

fn compile_environment_pointer(
    definition: &mir::ir::FunctionDefinition,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let payload_pointer = compile_payload_pointer(definition, types)?;

    Ok(if definition.is_thunk() {
        fmm::build::union_address(payload_pointer, 0)?.into()
    } else {
        payload_pointer
    })
}

fn compile_payload_pointer(
    definition: &mir::ir::FunctionDefinition,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    closure::compile_payload_pointer(fmm::build::bit_cast(
        fmm::types::Pointer::new(type_::compile_sized_closure(definition, types)),
        compile_untyped_closure_pointer(),
    ))
}

fn compile_closure_pointer(
    function_type: &mir::types::Function,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    Ok(fmm::build::bit_cast(
        fmm::types::Pointer::new(type_::compile_unsized_closure(function_type, types)),
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

    #[test]
    fn do_not_overwrite_global_functions_in_variables() {
        let function_type = mir::types::Function::new(vec![], mir::types::Type::Number);
        let context = Context::new(
            &mir::ir::Module::new(vec![], vec![], vec![], vec![], vec![]),
            CONFIGURATION.clone(),
        );

        compile(
            &context,
            &mir::ir::FunctionDefinition::new(
                "f",
                vec![],
                mir::ir::LetRecursive::new(
                    mir::ir::FunctionDefinition::new(
                        "g",
                        vec![],
                        mir::ir::Call::new(
                            function_type.clone(),
                            mir::ir::Variable::new("f"),
                            vec![],
                        ),
                        mir::types::Type::Number,
                    ),
                    mir::ir::Call::new(function_type.clone(), mir::ir::Variable::new("g"), vec![]),
                ),
                mir::types::Type::Number,
            ),
            true,
            &[(
                "f".into(),
                fmm::build::TypedExpression::new(
                    fmm::ir::Variable::new("f"),
                    fmm::types::Pointer::new(type_::compile_unsized_closure(
                        &function_type,
                        &Default::default(),
                    )),
                ),
            )]
            .into_iter()
            .collect(),
        )
        .unwrap();

        insta::assert_snapshot!(fmm::analysis::format_module(
            &context.module_builder().as_module()
        ));
    }
}
