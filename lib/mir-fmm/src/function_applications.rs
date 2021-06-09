use super::{
    closures, expressions, reference_count,
    types::{self, FUNCTION_ARGUMENT_OFFSET},
};
use crate::CompileError;
use std::collections::HashMap;

pub fn compile(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    closure_pointer: fmm::build::TypedExpression,
    arguments: &[fmm::build::TypedExpression],
    argument_types: &[&mir::types::Type],
    types: &HashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    compile_with_min_arity(
        module_builder,
        instruction_builder,
        closure_pointer,
        arguments,
        1,
        argument_types,
        types,
    )
}

fn compile_with_min_arity(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    closure_pointer: fmm::build::TypedExpression,
    arguments: &[fmm::build::TypedExpression],
    min_arity: usize,
    argument_types: &[&mir::types::Type],
    types: &HashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(if arguments.is_empty() {
        closure_pointer
    } else if arguments.len() < min_arity {
        compile_create_closure(
            module_builder,
            instruction_builder,
            closure_pointer,
            arguments,
            argument_types,
            types,
        )?
    } else if types::get_arity(get_entry_function_type(&closure_pointer)) == min_arity {
        compile_direct_call(instruction_builder, closure_pointer, arguments)?
    } else {
        instruction_builder.if_(
            fmm::build::comparison_operation(
                fmm::ir::ComparisonOperator::Equal,
                closures::compile_load_arity(instruction_builder, closure_pointer.clone())?,
                expressions::compile_arity(min_arity),
            )?,
            |instruction_builder| -> Result<_, CompileError> {
                Ok(instruction_builder.branch(compile(
                    module_builder,
                    &instruction_builder,
                    compile_direct_call(
                        &instruction_builder,
                        closure_pointer.clone(),
                        &arguments[..min_arity],
                    )?,
                    &arguments[min_arity..],
                    &argument_types[min_arity..],
                    types,
                )?))
            },
            |instruction_builder| {
                Ok(instruction_builder.branch(compile_with_min_arity(
                    module_builder,
                    &instruction_builder,
                    closure_pointer.clone(),
                    arguments,
                    min_arity + 1,
                    argument_types,
                    types,
                )?))
            },
        )?
    })
}

fn compile_direct_call(
    instruction_builder: &fmm::build::InstructionBuilder,
    closure_pointer: fmm::build::TypedExpression,
    arguments: &[fmm::build::TypedExpression],
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(instruction_builder.call(
        fmm::build::bit_cast(
            types::compile_curried_entry_function(
                get_entry_function_type(&closure_pointer),
                arguments.len(),
            ),
            closures::compile_load_entry_function(&instruction_builder, closure_pointer.clone())?,
        ),
        vec![
            fmm::build::bit_cast(types::compile_untyped_closure_pointer(), closure_pointer).into(),
        ]
        .into_iter()
        .chain(arguments.iter().cloned())
        .collect(),
    )?)
}

fn compile_create_closure(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    closure_pointer: fmm::build::TypedExpression,
    arguments: &[fmm::build::TypedExpression],
    argument_types: &[&mir::types::Type],
    types: &HashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let entry_function_type = get_entry_function_type(&closure_pointer);

    let target_entry_function_type = fmm::types::Function::new(
        entry_function_type.arguments()[..types::FUNCTION_ARGUMENT_OFFSET]
            .iter()
            .cloned()
            .chain(
                entry_function_type.arguments()
                    [arguments.len() + types::FUNCTION_ARGUMENT_OFFSET..]
                    .iter()
                    .cloned(),
            )
            .collect(),
        entry_function_type.result().clone(),
        fmm::types::CallingConvention::Source,
    );

    let closure = closures::compile_closure_content(
        compile_partially_applied_entry_function(
            module_builder,
            &target_entry_function_type,
            &closure_pointer.type_(),
            &arguments
                .iter()
                .map(|argument| argument.type_())
                .collect::<Vec<_>>(),
            argument_types,
            types,
        )?,
        closures::compile_drop_function_for_partially_applied_closure(
            module_builder,
            &closure_pointer.type_(),
            &arguments
                .iter()
                .map(|argument| argument.type_())
                .zip(argument_types.iter().cloned())
                .collect::<Vec<_>>(),
            types,
        )?,
        vec![closure_pointer]
            .into_iter()
            .chain(arguments.iter().cloned())
            .collect::<Vec<_>>(),
    );
    let closure_pointer =
        reference_count::allocate_heap(instruction_builder, closure.type_().clone())?;
    instruction_builder.store(closure, closure_pointer.clone());

    Ok(fmm::build::bit_cast(
        fmm::types::Pointer::new(types::compile_raw_closure(
            target_entry_function_type,
            types::compile_unsized_environment(),
        )),
        closure_pointer,
    )
    .into())
}

fn compile_partially_applied_entry_function(
    module_builder: &fmm::build::ModuleBuilder,
    entry_function_type: &fmm::types::Function,
    closure_pointer_type: &fmm::types::Type,
    argument_types: &[&fmm::types::Type],
    mir_argument_types: &[&mir::types::Type],
    types: &HashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let curried_entry_function_type =
        types::compile_curried_entry_function(&entry_function_type, 1);
    let arguments = curried_entry_function_type
        .arguments()
        .iter()
        .enumerate()
        .map(|(index, type_)| fmm::ir::Argument::new(format!("arg_{}", index), type_.clone()))
        .collect::<Vec<_>>();

    module_builder.define_anonymous_function(
        arguments.clone(),
        |instruction_builder| {
            let partially_applied_closure_pointer = fmm::build::bit_cast(
                fmm::types::Pointer::new(types::compile_raw_closure(
                    entry_function_type.clone(),
                    fmm::types::Record::new(
                        vec![closure_pointer_type.clone()]
                            .into_iter()
                            .chain(argument_types.iter().cloned().cloned())
                            .collect(),
                    ),
                )),
                fmm::build::variable(arguments[0].name(), arguments[0].type_().clone()),
            );
            let environment = instruction_builder.load(closures::compile_environment_pointer(
                partially_applied_closure_pointer.clone(),
            )?)?;
            let closure_pointer = instruction_builder.deconstruct_record(environment.clone(), 0)?;
            let arguments = (0..argument_types.len())
                .map(|index| instruction_builder.deconstruct_record(environment.clone(), index + 1))
                .chain(vec![Ok(fmm::build::variable(
                    arguments[FUNCTION_ARGUMENT_OFFSET].name(),
                    arguments[FUNCTION_ARGUMENT_OFFSET].type_().clone(),
                ))])
                .collect::<Result<Vec<_>, _>>()?;

            reference_count::clone_function(&instruction_builder, &closure_pointer)?;

            for (argument, type_) in arguments[..argument_types.len()]
                .iter()
                .zip(mir_argument_types)
            {
                reference_count::clone_expression(&instruction_builder, &argument, type_, types)?;
            }

            reference_count::drop_function(
                &instruction_builder,
                &partially_applied_closure_pointer.into(),
            )?;

            Ok(instruction_builder.return_(
                if types::get_arity(get_entry_function_type(&closure_pointer)) == arguments.len() {
                    compile_direct_call(&instruction_builder, closure_pointer, &arguments)?
                } else {
                    instruction_builder.if_(
                        fmm::build::comparison_operation(
                            fmm::ir::ComparisonOperator::Equal,
                            closures::compile_load_arity(
                                &instruction_builder,
                                closure_pointer.clone(),
                            )?,
                            expressions::compile_arity(arguments.len()),
                        )?,
                        |instruction_builder| -> Result<_, CompileError> {
                            Ok(instruction_builder.branch(compile_direct_call(
                                &instruction_builder,
                                closure_pointer.clone(),
                                &arguments,
                            )?))
                        },
                        |instruction_builder| {
                            Ok(instruction_builder.branch(compile_create_closure(
                                module_builder,
                                &instruction_builder,
                                closure_pointer.clone(),
                                &arguments,
                                mir_argument_types,
                                types,
                            )?))
                        },
                    )?
                },
            ))
        },
        curried_entry_function_type.result().clone(),
        fmm::types::CallingConvention::Source,
    )
}

fn get_entry_function_type(closure_pointer: &fmm::build::TypedExpression) -> &fmm::types::Function {
    closure_pointer
        .type_()
        .to_pointer()
        .unwrap()
        .element()
        .to_record()
        .unwrap()
        .elements()[0]
        .to_function()
        .unwrap()
}
