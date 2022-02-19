use super::error::CompileError;
use crate::{closures, expressions, reference_count, types};
use fnv::FnvHashMap;

const CLOSURE_NAME: &str = "_closure";
// TODO Inject this as a configuration.
const YIELD_FUNCTION_NAME: &str = "_pen_yield";

pub fn compile(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &mir::ir::Definition,
    global: bool,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(if definition.is_thunk() {
        compile_thunk(module_builder, definition, global, variables, types)?
    } else {
        compile_non_thunk(module_builder, definition, global, variables, types)?
    })
}

fn compile_non_thunk(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &mir::ir::Definition,
    global: bool,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    module_builder.define_anonymous_function(
        compile_arguments(definition, types),
        |instruction_builder| {
            Ok(instruction_builder.return_(compile_body(
                module_builder,
                &instruction_builder,
                definition,
                global,
                variables,
                types,
            )?))
        },
        types::compile(definition.result_type(), types),
        fmm::types::CallingConvention::Source,
    )
}

fn compile_thunk(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &mir::ir::Definition,
    global: bool,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    compile_initial_thunk_entry(
        module_builder,
        definition,
        global,
        compile_normal_thunk_entry(module_builder, definition, types)?,
        compile_locked_thunk_entry(module_builder, definition, types)?,
        variables,
        types,
    )
}

fn compile_body(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    definition: &mir::ir::Definition,
    global: bool,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let environment_pointer = compile_environment_pointer(definition, types)?;

    expressions::compile(
        module_builder,
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
                            reference_count::clone_expression(
                                instruction_builder,
                                &instruction_builder.load(fmm::build::record_address(
                                    environment_pointer.clone(),
                                    index,
                                )?)?,
                                free_variable.type_(),
                                types,
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
                    compile_closure_pointer(definition.type_(), types)?,
                )]
            })
            .chain(definition.arguments().iter().map(|argument| {
                (
                    argument.name().into(),
                    fmm::build::variable(argument.name(), types::compile(argument.type_(), types)),
                )
            }))
            .collect(),
        types,
    )
}

fn compile_initial_thunk_entry(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &mir::ir::Definition,
    global: bool,
    normal_entry_function: fmm::build::TypedExpression,
    lock_entry_function: fmm::build::TypedExpression,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let entry_function_name = module_builder.generate_name();
    let entry_function_type = types::compile_entry_function(definition.type_(), types);
    let arguments = compile_arguments(definition, types);

    module_builder.define_function(
        &entry_function_name,
        arguments.clone(),
        |instruction_builder| {
            let entry_function_pointer = compile_entry_function_pointer(definition, types)?;

            instruction_builder.if_(
                instruction_builder.compare_and_swap(
                    entry_function_pointer.clone(),
                    fmm::build::variable(&entry_function_name, entry_function_type.clone()),
                    lock_entry_function.clone(),
                    fmm::ir::AtomicOrdering::Acquire,
                    fmm::ir::AtomicOrdering::Relaxed,
                ),
                |instruction_builder| -> Result<_, CompileError> {
                    let closure = reference_count::clone_expression(
                        &instruction_builder,
                        &compile_closure_pointer(definition.type_(), types)?,
                        &definition.type_().clone().into(),
                        types,
                    )?;

                    let value = compile_body(
                        module_builder,
                        &instruction_builder,
                        definition,
                        global,
                        variables,
                        types,
                    )?;

                    let environment_pointer = compile_environment_pointer(definition, types)?;

                    // TODO Remove these extra drops of free variables when we move them in function
                    // bodies rather than cloning them.
                    // See also https://github.com/pen-lang/pen/issues/295.
                    for (index, free_variable) in definition.environment().iter().enumerate() {
                        reference_count::drop_expression(
                            &instruction_builder,
                            &instruction_builder.load(fmm::build::record_address(
                                environment_pointer.clone(),
                                index,
                            )?)?,
                            free_variable.type_(),
                            types,
                        )?;
                    }

                    instruction_builder.store(
                        reference_count::clone_expression(
                            &instruction_builder,
                            &value,
                            definition.result_type(),
                            types,
                        )?,
                        compile_thunk_value_pointer(definition, types)?,
                    );

                    instruction_builder.store(
                        closures::compile_normal_thunk_drop_function(
                            module_builder,
                            definition,
                            types,
                        )?,
                        compile_drop_function_pointer(definition, types)?,
                    );

                    instruction_builder.atomic_store(
                        normal_entry_function.clone(),
                        entry_function_pointer.clone(),
                        fmm::ir::AtomicOrdering::Release,
                    );

                    reference_count::drop_expression(
                        &instruction_builder,
                        &closure,
                        &definition.type_().clone().into(),
                        types,
                    )?;

                    Ok(instruction_builder.return_(value))
                },
                |instruction_builder| {
                    Ok(instruction_builder.return_(instruction_builder.call(
                        instruction_builder.atomic_load(
                            compile_entry_function_pointer(definition, types)?,
                            fmm::ir::AtomicOrdering::Acquire,
                        )?,
                        compile_argument_variables(&arguments),
                    )?))
                },
            )?;

            Ok(instruction_builder.unreachable())
        },
        types::compile(definition.result_type(), types),
        fmm::types::CallingConvention::Source,
        fmm::ir::Linkage::Internal,
    )
}

fn compile_normal_thunk_entry(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &mir::ir::Definition,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    module_builder.define_anonymous_function(
        compile_arguments(definition, types),
        |instruction_builder| compile_normal_body(&instruction_builder, definition, types),
        types::compile(definition.result_type(), types),
        fmm::types::CallingConvention::Source,
    )
}

fn compile_locked_thunk_entry(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &mir::ir::Definition,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let entry_function_name = module_builder.generate_name();
    let entry_function = fmm::build::variable(
        &entry_function_name,
        types::compile_entry_function(definition.type_(), types),
    );
    let arguments = compile_arguments(definition, types);

    module_builder.define_function(
        &entry_function_name,
        arguments.clone(),
        |instruction_builder| {
            instruction_builder.if_(
                fmm::build::comparison_operation(
                    fmm::ir::ComparisonOperator::Equal,
                    fmm::build::bit_cast(
                        fmm::types::Primitive::PointerInteger,
                        instruction_builder.atomic_load(
                            compile_entry_function_pointer(definition, types)?,
                            fmm::ir::AtomicOrdering::Acquire,
                        )?,
                    ),
                    fmm::build::bit_cast(
                        fmm::types::Primitive::PointerInteger,
                        entry_function.clone(),
                    ),
                )?,
                |instruction_builder| {
                    instruction_builder.call(
                        fmm::build::variable(
                            YIELD_FUNCTION_NAME,
                            fmm::types::Function::new(
                                vec![],
                                fmm::types::VOID_TYPE.clone(),
                                fmm::types::CallingConvention::Source,
                            ),
                        ),
                        vec![],
                    )?;

                    Ok(instruction_builder.return_(instruction_builder.call(
                        entry_function.clone(),
                        compile_argument_variables(&arguments),
                    )?))
                },
                |instruction_builder| compile_normal_body(&instruction_builder, definition, types),
            )?;

            Ok(instruction_builder.unreachable())
        },
        types::compile(definition.result_type(), types),
        fmm::types::CallingConvention::Source,
        fmm::ir::Linkage::Internal,
    )
}

fn compile_normal_body(
    instruction_builder: &fmm::build::InstructionBuilder,
    definition: &mir::ir::Definition,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::ir::Block, CompileError> {
    let value = reference_count::clone_expression(
        instruction_builder,
        &instruction_builder.load(compile_thunk_value_pointer(definition, types)?)?,
        definition.result_type(),
        types,
    )?;

    reference_count::drop_expression(
        instruction_builder,
        &compile_closure_pointer(definition.type_(), types)?,
        &definition.type_().clone().into(),
        types,
    )?;

    Ok(instruction_builder.return_(value))
}

fn compile_entry_function_pointer(
    definition: &mir::ir::Definition,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    closures::compile_entry_function_pointer(compile_closure_pointer(definition.type_(), types)?)
}

fn compile_drop_function_pointer(
    definition: &mir::ir::Definition,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    closures::compile_drop_function_pointer(compile_closure_pointer(definition.type_(), types)?)
}

fn compile_arguments(
    definition: &mir::ir::Definition,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Vec<fmm::ir::Argument> {
    [fmm::ir::Argument::new(
        CLOSURE_NAME,
        types::compile_untyped_closure_pointer(),
    )]
    .into_iter()
    .chain(definition.arguments().iter().map(|argument| {
        fmm::ir::Argument::new(argument.name(), types::compile(argument.type_(), types))
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
    definition: &mir::ir::Definition,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::union_address(compile_payload_pointer(definition, types)?, 1)?.into())
}

fn compile_environment_pointer(
    definition: &mir::ir::Definition,
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
    definition: &mir::ir::Definition,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    closures::compile_payload_pointer(fmm::build::bit_cast(
        fmm::types::Pointer::new(types::compile_sized_closure(definition, types)),
        compile_untyped_closure_pointer(),
    ))
}

fn compile_closure_pointer(
    function_type: &mir::types::Function,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    Ok(fmm::build::bit_cast(
        fmm::types::Pointer::new(types::compile_unsized_closure(function_type, types)),
        compile_untyped_closure_pointer(),
    )
    .into())
}

fn compile_untyped_closure_pointer() -> fmm::build::TypedExpression {
    fmm::build::variable(CLOSURE_NAME, types::compile_untyped_closure_pointer())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn do_not_overwrite_global_functions_in_variables() {
        let function_type = mir::types::Function::new(vec![], mir::types::Type::Number);
        let module_builder = fmm::build::ModuleBuilder::new();

        compile(
            &module_builder,
            &mir::ir::Definition::new(
                "f",
                vec![],
                mir::ir::LetRecursive::new(
                    mir::ir::Definition::new(
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
                    fmm::types::Pointer::new(types::compile_unsized_closure(
                        &function_type,
                        &Default::default(),
                    )),
                ),
            )]
            .into_iter()
            .collect(),
            &Default::default(),
        )
        .unwrap();

        insta::assert_snapshot!(fmm::analysis::format_module(&module_builder.as_module()));
    }
}
