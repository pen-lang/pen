use super::error::CompileError;
use crate::{
    calls, closures, context::Context, entry_functions, pointers, records, reference_count, types,
    variants,
};
use fnv::FnvHashMap;

pub fn compile(
    context: &Context,
    instruction_builder: &fmm::build::InstructionBuilder,
    expression: &mir::ir::Expression,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let compile = |expression, variables: &FnvHashMap<_, _>| {
        compile(context, instruction_builder, expression, variables)
    };

    Ok(match expression {
        mir::ir::Expression::ArithmeticOperation(operation) => {
            compile_arithmetic_operation(context, instruction_builder, operation, variables)?.into()
        }
        mir::ir::Expression::Boolean(boolean) => fmm::ir::Primitive::Boolean(*boolean).into(),
        mir::ir::Expression::Case(case) => {
            compile_case(context, instruction_builder, case, variables)?
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
                                    context.types(),
                                )?,
                            ))
                        })
                        .collect::<Result<Vec<_>, CompileError>>()?,
                )
                .collect(),
        )?,
        mir::ir::Expression::ComparisonOperation(operation) => {
            compile_comparison_operation(context, instruction_builder, operation, variables)?.into()
        }
        mir::ir::Expression::DiscardHeap(discard) => {
            for id in discard.ids() {
                let pointer = variables[id].clone();

                instruction_builder.if_(
                    pointers::equal(
                        pointer.clone(),
                        fmm::ir::Undefined::new(pointer.type_().clone()),
                    )?,
                    |builder| -> Result<_, CompileError> {
                        Ok(builder.branch(fmm::ir::VOID_VALUE.clone()))
                    },
                    |builder| {
                        reference_count::free_heap(instruction_builder, pointer.clone())?;

                        Ok(builder.branch(fmm::ir::VOID_VALUE.clone()))
                    },
                )?;
            }

            compile(discard.expression(), variables)?
        }
        mir::ir::Expression::DropVariables(drop) => {
            for (variable, type_) in drop.variables() {
                reference_count::drop_expression(
                    instruction_builder,
                    &variables[variable],
                    type_,
                    context.types(),
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
        mir::ir::Expression::If(if_) => compile_if(context, instruction_builder, if_, variables)?,
        mir::ir::Expression::Let(let_) => {
            compile_let(context, instruction_builder, let_, variables)?
        }
        mir::ir::Expression::LetRecursive(let_) => {
            compile_let_recursive(context, instruction_builder, let_, variables)?
        }
        mir::ir::Expression::None => fmm::ir::Undefined::new(types::compile_none()).into(),
        mir::ir::Expression::Number(number) => fmm::ir::Primitive::Float64(*number).into(),
        mir::ir::Expression::Record(record) => {
            compile_record(context, instruction_builder, record, variables)?
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
                context.types(),
            )?;

            let field = reference_count::clone_expression(
                instruction_builder,
                &field,
                &context.types()[record_type.name()].fields()[field_index],
                context.types(),
            )?;
            reference_count::drop_expression(
                instruction_builder,
                &record,
                &record_type.into(),
                context.types(),
            )?;

            field
        }
        mir::ir::Expression::RetainHeap(retain) => {
            let mut reused_variables = FnvHashMap::default();

            for (name, type_) in retain.drop().variables() {
                if retain.ids().contains_key(name) {
                    reused_variables.insert(
                        name,
                        reference_count::drop_or_reuse_expression(
                            instruction_builder,
                            &variables[name],
                            type_,
                            context.types(),
                        )?,
                    );
                } else {
                    reference_count::drop_expression(
                        instruction_builder,
                        &variables[name],
                        type_,
                        context.types(),
                    )?;
                }
            }

            compile(
                retain.drop().expression(),
                &variables
                    .clone()
                    .into_iter()
                    .chain(
                        retain
                            .ids()
                            .iter()
                            .map(|(name, id)| (id.clone(), reused_variables.remove(name).unwrap())),
                    )
                    .collect(),
            )?
        }
        mir::ir::Expression::ReuseRecord(reuse) => {
            let pointer_type = fmm::types::GENERIC_POINTER_TYPE.clone();
            let pointer = variables[reuse.id()].clone();

            instruction_builder.if_(
                pointers::equal(pointer.clone(), fmm::ir::Undefined::new(pointer_type))?,
                |builder| -> Result<_, CompileError> {
                    Ok(builder.branch(compile_record(
                        context,
                        &builder,
                        reuse.record(),
                        variables,
                    )?))
                },
                |builder| {
                    Ok(builder.branch(compile_boxed_record_with_pointer(
                        context,
                        &builder,
                        pointer.clone(),
                        reuse.record(),
                        variables,
                    )?))
                },
            )?
        }
        mir::ir::Expression::ByteString(string) => {
            if string.value().is_empty() {
                fmm::ir::Undefined::new(types::compile_string()).into()
            } else {
                reference_count::compile_tagged_pointer(
                    &fmm::build::bit_cast(
                        types::compile_string(),
                        context.module_builder().define_anonymous_variable(
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
        mir::ir::Expression::TryOperation(operation) => {
            compile_try_operation(context, instruction_builder, operation, variables)?
        }
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
    context: &Context,
    instruction_builder: &fmm::build::InstructionBuilder,
    if_: &mir::ir::If,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let compile = |instruction_builder: &fmm::build::InstructionBuilder, expression| {
        compile(context, instruction_builder, expression, variables)
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
    context: &Context,
    instruction_builder: &fmm::build::InstructionBuilder,
    case: &mir::ir::Case,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(compile_alternatives(
        context,
        instruction_builder,
        compile(context, instruction_builder, case.argument(), variables)?,
        case.alternatives(),
        case.default_alternative(),
        variables,
    )?
    .unwrap())
}

fn compile_alternatives(
    context: &Context,
    instruction_builder: &fmm::build::InstructionBuilder,
    argument: fmm::build::TypedExpression,
    alternatives: &[mir::ir::Alternative],
    default_alternative: Option<&mir::ir::DefaultAlternative>,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<Option<fmm::build::TypedExpression>, CompileError> {
    Ok(match alternatives {
        [] => default_alternative
            .map(|alternative| {
                compile(
                    context,
                    instruction_builder,
                    alternative.expression(),
                    &variables
                        .clone()
                        .into_iter()
                        .chain([(alternative.name().into(), argument)])
                        .collect(),
                )
            })
            .transpose()?,
        [alternative, ..] => Some(instruction_builder.if_(
            compile_tag_comparison(instruction_builder, &argument, alternative.type_())?,
            |instruction_builder| -> Result<_, CompileError> {
                Ok(instruction_builder.branch(compile(
                    context,
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
                                context.types(),
                            )?,
                        )])
                        .collect(),
                )?))
            },
            |instruction_builder| {
                Ok(
                    if let Some(expression) = compile_alternatives(
                        context,
                        &instruction_builder,
                        argument.clone(),
                        &alternatives[1..],
                        default_alternative,
                        variables,
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
    Ok(pointers::equal(
        instruction_builder.deconstruct_record(argument.clone(), 0)?,
        variants::compile_tag(type_),
    )?)
}

fn compile_let(
    context: &Context,
    instruction_builder: &fmm::build::InstructionBuilder,
    let_: &mir::ir::Let,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let compile =
        |expression, variables| compile(context, instruction_builder, expression, variables);

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
    context: &Context,
    instruction_builder: &fmm::build::InstructionBuilder,
    let_: &mir::ir::LetRecursive,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let closure_pointer = reference_count::allocate_heap(
        instruction_builder,
        types::compile_sized_closure(let_.definition(), context.types()),
    )?;

    instruction_builder.store(
        closures::compile_closure_content(
            entry_functions::compile(context, let_.definition(), false, variables)?,
            closures::compile_drop_function(context, let_.definition())?,
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
                        types::compile_thunk_payload(let_.definition(), context.types()),
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
        context,
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
                        context.types(),
                    )),
                    closure_pointer,
                )
                .into(),
            )])
            .collect(),
    )
}

fn compile_arithmetic_operation(
    context: &Context,
    instruction_builder: &fmm::build::InstructionBuilder,
    operation: &mir::ir::ArithmeticOperation,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::ir::ArithmeticOperation, CompileError> {
    let compile = |expression| compile(context, instruction_builder, expression, variables);

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
    context: &Context,
    instruction_builder: &fmm::build::InstructionBuilder,
    operation: &mir::ir::ComparisonOperation,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::ir::ComparisonOperation, CompileError> {
    let compile = |expression| compile(context, instruction_builder, expression, variables);

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

fn compile_record(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    record: &mir::ir::Record,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(if types::is_record_boxed(record.type_(), context.types()) {
        compile_boxed_record_with_pointer(
            context,
            builder,
            reference_count::allocate_heap(
                builder,
                types::compile_unboxed_record(record.type_(), context.types()),
            )?,
            record,
            variables,
        )?
    } else {
        compile_unboxed_record(context, builder, record, variables)?.into()
    })
}

fn compile_boxed_record_with_pointer(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    pointer: fmm::build::TypedExpression,
    record: &mir::ir::Record,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let unboxed = compile_unboxed_record(context, builder, record, variables)?;

    builder.store(
        unboxed.clone(),
        fmm::build::bit_cast(
            fmm::types::Pointer::new(unboxed.type_().clone()),
            pointer.clone(),
        ),
    );

    Ok(fmm::build::bit_cast(
        types::compile_record(record.type_(), context.types()),
        pointer,
    )
    .into())
}

fn compile_unboxed_record(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    record: &mir::ir::Record,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::ir::Record, CompileError> {
    Ok(fmm::build::record(
        record
            .fields()
            .iter()
            .map(|argument| compile(context, builder, argument, variables))
            .collect::<Result<_, _>>()?,
    ))
}

fn compile_try_operation(
    context: &Context,
    instruction_builder: &fmm::build::InstructionBuilder,
    operation: &mir::ir::TryOperation,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let operand = compile(context, instruction_builder, operation.operand(), variables)?;

    instruction_builder.if_(
        compile_tag_comparison(instruction_builder, &operand, operation.type_())?,
        |instruction_builder| -> Result<_, CompileError> {
            Ok(instruction_builder.return_(compile(
                context,
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
                            context.types(),
                        )?,
                    )])
                    .collect(),
            )?))
        },
        |instruction_builder| Ok(instruction_builder.branch(operand.clone())),
    )
}
