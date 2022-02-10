use super::error::CompileError;
use crate::{calls, closures, entry_functions, records, reference_count, types, variants};
use fnv::FnvHashMap;

pub fn compile(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    expression: &mir::ir::Expression,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let compile = |expression, variables: &FnvHashMap<_, _>| {
        compile(
            module_builder,
            instruction_builder,
            expression,
            variables,
            types,
        )
    };

    Ok(match expression {
        mir::ir::Expression::ArithmeticOperation(operation) => compile_arithmetic_operation(
            module_builder,
            instruction_builder,
            operation,
            variables,
            types,
        )?
        .into(),
        mir::ir::Expression::Boolean(boolean) => fmm::ir::Primitive::Boolean(*boolean).into(),
        mir::ir::Expression::Case(case) => {
            compile_case(module_builder, instruction_builder, case, variables, types)?
        }
        mir::ir::Expression::CloneVariables(clone) => compile(
            clone.expression(),
            &variables
                .iter()
                .map(|(name, expression)| (name.clone(), expression.clone()))
                .chain(
                    clone
                        .variables()
                        .iter()
                        .map(|(variable, type_)| {
                            Ok((
                                variable.into(),
                                reference_count::clone_expression(
                                    instruction_builder,
                                    &variables[variable],
                                    type_,
                                    types,
                                )?,
                            ))
                        })
                        .collect::<Result<Vec<_>, CompileError>>()?,
                )
                .collect(),
        )?,
        mir::ir::Expression::ComparisonOperation(operation) => compile_comparison_operation(
            module_builder,
            instruction_builder,
            operation,
            variables,
            types,
        )?
        .into(),
        mir::ir::Expression::DropVariables(drop) => {
            for (variable, type_) in drop.variables() {
                reference_count::drop_expression(
                    instruction_builder,
                    &variables[variable],
                    type_,
                    types,
                )?;
            }

            compile(drop.expression(), variables)?
        }
        mir::ir::Expression::Call(call) => calls::compile(
            instruction_builder,
            &compile(call.function(), variables)?,
            &call
                .arguments()
                .iter()
                .map(|argument| compile(argument, variables))
                .collect::<Result<Vec<_>, CompileError>>()?,
        )?,
        mir::ir::Expression::If(if_) => {
            compile_if(module_builder, instruction_builder, if_, variables, types)?
        }
        mir::ir::Expression::Let(let_) => {
            compile_let(module_builder, instruction_builder, let_, variables, types)?
        }
        mir::ir::Expression::LetRecursive(let_) => {
            compile_let_recursive(module_builder, instruction_builder, let_, variables, types)?
        }
        mir::ir::Expression::None => fmm::ir::Undefined::new(types::compile_none()).into(),
        mir::ir::Expression::Number(number) => fmm::ir::Primitive::Float64(*number).into(),
        mir::ir::Expression::Record(record) => {
            let unboxed = fmm::build::record(
                record
                    .fields()
                    .iter()
                    .map(|argument| compile(argument, variables))
                    .collect::<Result<_, _>>()?,
            );

            if types::is_record_boxed(record.type_(), types) {
                let pointer =
                    reference_count::allocate_heap(instruction_builder, unboxed.type_().clone())?;

                instruction_builder.store(unboxed, pointer.clone());

                fmm::build::bit_cast(types::compile_record(record.type_(), types), pointer).into()
            } else {
                unboxed.into()
            }
        }
        mir::ir::Expression::RecordField(field) => {
            let record_type = field.type_().clone();
            let field_index = field.index();

            let record = compile(field.record(), variables)?;
            let field = records::get_record_field(
                instruction_builder,
                &record,
                field.type_(),
                field.index(),
                types,
            )?;

            let field = reference_count::clone_expression(
                instruction_builder,
                &field,
                &types[record_type.name()].fields()[field_index],
                types,
            )?;
            reference_count::drop_expression(
                instruction_builder,
                &record,
                &record_type.into(),
                types,
            )?;

            field
        }
        mir::ir::Expression::ByteString(string) => {
            if string.value().is_empty() {
                fmm::ir::Undefined::new(types::compile_string()).into()
            } else {
                reference_count::compile_tagged_pointer(
                    &fmm::build::bit_cast(
                        types::compile_string(),
                        module_builder.define_anonymous_variable(
                            fmm::build::record(
                                [
                                    fmm::ir::Primitive::PointerInteger(string.value().len() as i64)
                                        .into(),
                                ]
                                .into_iter()
                                .chain(
                                    string
                                        .value()
                                        .iter()
                                        .map(|&byte| fmm::ir::Primitive::Integer8(byte).into()),
                                )
                                .collect(),
                            ),
                            false,
                            None,
                        ),
                    )
                    .into(),
                )?
            }
        }
        mir::ir::Expression::TryOperation(operation) => compile_try_operation(
            module_builder,
            instruction_builder,
            operation,
            variables,
            types,
        )?,
        mir::ir::Expression::Variable(variable) => variables[variable.name()].clone(),
        mir::ir::Expression::Variant(variant) => fmm::build::record(vec![
            variants::compile_tag(variant.type_()),
            variants::compile_boxed_payload(
                instruction_builder,
                &compile(variant.payload(), variables)?,
            )?,
        ])
        .into(),
    })
}

fn compile_if(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    if_: &mir::ir::If,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let compile = |instruction_builder: &fmm::build::InstructionBuilder, expression| {
        compile(
            module_builder,
            instruction_builder,
            expression,
            variables,
            types,
        )
    };

    instruction_builder.if_(
        compile(instruction_builder, if_.condition())?,
        |instruction_builder| {
            Ok(instruction_builder.branch(compile(&instruction_builder, if_.then())?))
        },
        |instruction_builder| {
            Ok(instruction_builder.branch(compile(&instruction_builder, if_.else_())?))
        },
    )
}

fn compile_case(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    case: &mir::ir::Case,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(compile_alternatives(
        module_builder,
        instruction_builder,
        compile(
            module_builder,
            instruction_builder,
            case.argument(),
            variables,
            types,
        )?,
        case.alternatives(),
        case.default_alternative(),
        variables,
        types,
    )?
    .unwrap())
}

fn compile_alternatives(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    argument: fmm::build::TypedExpression,
    alternatives: &[mir::ir::Alternative],
    default_alternative: Option<&mir::ir::DefaultAlternative>,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<Option<fmm::build::TypedExpression>, CompileError> {
    Ok(match alternatives {
        [] => default_alternative
            .map(|alternative| {
                compile(
                    module_builder,
                    instruction_builder,
                    alternative.expression(),
                    &variables
                        .clone()
                        .into_iter()
                        .chain([(alternative.name().into(), argument)])
                        .collect(),
                    types,
                )
            })
            .transpose()?,
        [alternative, ..] => Some(instruction_builder.if_(
            compile_tag_comparison(instruction_builder, &argument, alternative.type_())?,
            |instruction_builder| -> Result<_, CompileError> {
                Ok(instruction_builder.branch(compile(
                    module_builder,
                    &instruction_builder,
                    alternative.expression(),
                    &variables
                        .clone()
                        .into_iter()
                        .chain([(
                            alternative.name().into(),
                            variants::compile_unboxed_payload(
                                &instruction_builder,
                                &instruction_builder.deconstruct_record(argument.clone(), 1)?,
                                alternative.type_(),
                                types,
                            )?,
                        )])
                        .collect(),
                    types,
                )?))
            },
            |instruction_builder| {
                Ok(
                    if let Some(expression) = compile_alternatives(
                        module_builder,
                        &instruction_builder,
                        argument.clone(),
                        &alternatives[1..],
                        default_alternative,
                        variables,
                        types,
                    )? {
                        instruction_builder.branch(expression)
                    } else {
                        instruction_builder.unreachable()
                    },
                )
            },
        )?),
    })
}

fn compile_tag_comparison(
    instruction_builder: &fmm::build::InstructionBuilder,
    argument: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::comparison_operation(
        fmm::ir::ComparisonOperator::Equal,
        fmm::build::bit_cast(
            fmm::types::Primitive::PointerInteger,
            instruction_builder.deconstruct_record(argument.clone(), 0)?,
        ),
        fmm::build::bit_cast(
            fmm::types::Primitive::PointerInteger,
            variants::compile_tag(type_),
        ),
    )?
    .into())
}

fn compile_let(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    let_: &mir::ir::Let,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let compile = |expression, variables| {
        compile(
            module_builder,
            instruction_builder,
            expression,
            variables,
            types,
        )
    };

    compile(
        let_.expression(),
        &variables
            .iter()
            .map(|(name, expression)| (name.clone(), expression.clone()))
            .chain([(
                let_.name().into(),
                compile(let_.bound_expression(), variables)?,
            )])
            .collect(),
    )
}

fn compile_let_recursive(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    let_: &mir::ir::LetRecursive,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let closure_pointer = reference_count::allocate_heap(
        instruction_builder,
        types::compile_sized_closure(let_.definition(), types),
    )?;

    instruction_builder.store(
        closures::compile_closure_content(
            entry_functions::compile(module_builder, let_.definition(), false, variables, types)?,
            closures::compile_drop_function(module_builder, let_.definition(), types)?,
            {
                let environment = fmm::build::record(
                    let_.definition()
                        .environment()
                        .iter()
                        .map(|free_variable| variables[free_variable.name()].clone())
                        .collect(),
                );

                if let_.definition().is_thunk() {
                    fmm::build::TypedExpression::from(fmm::ir::Union::new(
                        types::compile_thunk_payload(let_.definition(), types),
                        0,
                        environment,
                    ))
                } else {
                    environment.into()
                }
            },
        ),
        closure_pointer.clone(),
    );

    compile(
        module_builder,
        instruction_builder,
        let_.expression(),
        &variables
            .clone()
            .into_iter()
            .chain([(
                let_.definition().name().into(),
                fmm::build::bit_cast(
                    fmm::types::Pointer::new(types::compile_unsized_closure(
                        let_.definition().type_(),
                        types,
                    )),
                    closure_pointer,
                )
                .into(),
            )])
            .collect(),
        types,
    )
}

fn compile_arithmetic_operation(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    operation: &mir::ir::ArithmeticOperation,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::ir::ArithmeticOperation, CompileError> {
    let compile = |expression| {
        compile(
            module_builder,
            instruction_builder,
            expression,
            variables,
            types,
        )
    };

    let lhs = compile(operation.lhs())?;
    let rhs = compile(operation.rhs())?;

    Ok(match operation.operator() {
        mir::ir::ArithmeticOperator::Add => {
            fmm::build::arithmetic_operation(fmm::ir::ArithmeticOperator::Add, lhs, rhs)?
        }
        mir::ir::ArithmeticOperator::Subtract => {
            fmm::build::arithmetic_operation(fmm::ir::ArithmeticOperator::Subtract, lhs, rhs)?
        }
        mir::ir::ArithmeticOperator::Multiply => {
            fmm::build::arithmetic_operation(fmm::ir::ArithmeticOperator::Multiply, lhs, rhs)?
        }
        mir::ir::ArithmeticOperator::Divide => {
            fmm::build::arithmetic_operation(fmm::ir::ArithmeticOperator::Divide, lhs, rhs)?
        }
    })
}

fn compile_comparison_operation(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    operation: &mir::ir::ComparisonOperation,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::ir::ComparisonOperation, CompileError> {
    let compile = |expression| {
        compile(
            module_builder,
            instruction_builder,
            expression,
            variables,
            types,
        )
    };

    let lhs = compile(operation.lhs())?;
    let rhs = compile(operation.rhs())?;

    Ok(fmm::build::comparison_operation(
        match operation.operator() {
            mir::ir::ComparisonOperator::Equal => fmm::ir::ComparisonOperator::Equal,
            mir::ir::ComparisonOperator::NotEqual => fmm::ir::ComparisonOperator::NotEqual,
            mir::ir::ComparisonOperator::GreaterThan => fmm::ir::ComparisonOperator::GreaterThan,
            mir::ir::ComparisonOperator::GreaterThanOrEqual => {
                fmm::ir::ComparisonOperator::GreaterThanOrEqual
            }
            mir::ir::ComparisonOperator::LessThan => fmm::ir::ComparisonOperator::LessThan,
            mir::ir::ComparisonOperator::LessThanOrEqual => {
                fmm::ir::ComparisonOperator::LessThanOrEqual
            }
        },
        lhs,
        rhs,
    )?)
}

fn compile_try_operation(
    module_builder: &fmm::build::ModuleBuilder,
    instruction_builder: &fmm::build::InstructionBuilder,
    operation: &mir::ir::TryOperation,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let operand = compile(
        module_builder,
        instruction_builder,
        operation.operand(),
        variables,
        types,
    )?;

    instruction_builder.if_(
        compile_tag_comparison(instruction_builder, &operand, operation.type_())?,
        |instruction_builder| -> Result<_, CompileError> {
            Ok(instruction_builder.return_(compile(
                module_builder,
                &instruction_builder,
                operation.then(),
                &variables
                    .clone()
                    .into_iter()
                    .chain([(
                        operation.name().into(),
                        variants::compile_unboxed_payload(
                            &instruction_builder,
                            &instruction_builder.deconstruct_record(operand.clone(), 1)?,
                            operation.type_(),
                            types,
                        )?,
                    )])
                    .collect(),
                types,
            )?))
        },
        |instruction_builder| Ok(instruction_builder.branch(operand.clone())),
    )
}
