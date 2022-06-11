use super::error::CompileError;
use crate::{
    call, closure, context::Context, entry_function, pointer, record, reference_count, type_,
    variant,
};
use fnv::FnvHashMap;

pub fn compile(
    context: &Context,
    instruction_builder: &fmm::build::InstructionBuilder,
    expression: &mir::ir::Expression,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let compile =
        |expression, variables: &_| compile(context, instruction_builder, expression, variables);

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
                                reference_count::clone(
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
        mir::ir::Expression::DropVariables(drop) => {
            for (variable, type_) in drop.variables() {
                reference_count::drop(
                    instruction_builder,
                    &variables[variable],
                    type_,
                    context.types(),
                )?;
            }

            compile(drop.expression(), variables)?
        }
        mir::ir::Expression::Call(call) => call::compile(
            instruction_builder,
            &compile(call.function(), variables)?,
            &call
                .arguments()
                .iter()
                .map(|argument| compile(argument, variables))
                .collect::<Result<Vec<_>, _>>()?,
        )?,
        mir::ir::Expression::If(if_) => compile_if(context, instruction_builder, if_, variables)?,
        mir::ir::Expression::Let(let_) => {
            compile_let(context, instruction_builder, let_, variables)?
        }
        mir::ir::Expression::LetRecursive(let_) => {
            compile_let_recursive(context, instruction_builder, let_, variables)?
        }
        mir::ir::Expression::Synchronize(mark) => {
            compile_synchronize(context, instruction_builder, mark, variables)?
        }
        mir::ir::Expression::None => fmm::ir::Undefined::new(type_::compile_none()).into(),
        mir::ir::Expression::Number(number) => fmm::ir::Primitive::Float64(*number).into(),
        mir::ir::Expression::Record(record) => {
            compile_record(context, instruction_builder, record, variables)?
        }
        mir::ir::Expression::RecordField(field) => {
            let record_type = field.type_().clone();
            let field_index = field.index();

            let record = compile(field.record(), variables)?;
            let field = record::get_field(
                context,
                instruction_builder,
                &record,
                field.type_(),
                field.index(),
            )?;

            let field = reference_count::clone(
                instruction_builder,
                &field,
                &context.types()[record_type.name()].fields()[field_index],
                context.types(),
            )?;
            reference_count::drop(
                instruction_builder,
                &record,
                &record_type.into(),
                context.types(),
            )?;

            field
        }
        mir::ir::Expression::RecordUpdate(update) => {
            let record = compile(update.record(), variables)?;
            let record_body_type = &context.types()[update.type_().name()];
            let fields = update
                .fields()
                .iter()
                .map(|field| Ok((field.index(), compile(field.expression(), variables)?)))
                .collect::<Result<FnvHashMap<_, _>, CompileError>>()?;

            let compile_unboxed = |builder: &_, clone: bool| -> Result<_, CompileError> {
                Ok(fmm::build::record(
                    record_body_type
                        .fields()
                        .iter()
                        .enumerate()
                        .map(|(index, field_type)| -> Result<_, CompileError> {
                            let field = record::get_field(
                                context,
                                builder,
                                &record,
                                update.type_(),
                                index,
                            )?;

                            Ok(if let Some(expression) = fields.get(&index) {
                                if !clone {
                                    reference_count::drop(
                                        builder,
                                        &field,
                                        field_type,
                                        context.types(),
                                    )?;
                                }

                                expression.clone()
                            } else if clone {
                                reference_count::clone(
                                    builder,
                                    &field,
                                    field_type,
                                    context.types(),
                                )?
                            } else {
                                field
                            })
                        })
                        .collect::<Result<_, _>>()?,
                ))
            };

            if type_::is_record_boxed(update.type_(), context.types()) {
                instruction_builder.if_(
                    reference_count::pointer::is_owned(instruction_builder, &record)?,
                    |builder| -> Result<_, CompileError> {
                        Ok(builder.branch(compile_boxed_record(
                            &builder,
                            record.clone(),
                            compile_unboxed(&builder, false)?,
                        )?))
                    },
                    |builder| {
                        let updated_record = compile_boxed_record(
                            &builder,
                            allocate_record_heap(context, &builder, update.type_())?,
                            compile_unboxed(&builder, true)?,
                        )?;

                        reference_count::drop(
                            &builder,
                            &record,
                            &update.type_().clone().into(),
                            context.types(),
                        )?;

                        Ok(builder.branch(updated_record))
                    },
                )?
            } else {
                compile_unboxed(instruction_builder, false)?.into()
            }
        }
        mir::ir::Expression::ByteString(string) => {
            if string.value().is_empty() {
                fmm::ir::Undefined::new(type_::compile_string()).into()
            } else {
                reference_count::pointer::tag_as_static(
                    &fmm::build::bit_cast(
                        type_::compile_string(),
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
        mir::ir::Expression::Variant(variant) => variant::upcast(
            context,
            instruction_builder,
            &compile(variant.payload(), variables)?,
            variant.type_(),
        )?,
    })
}

fn compile_if(
    context: &Context,
    instruction_builder: &fmm::build::InstructionBuilder,
    if_: &mir::ir::If,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let compile = |instruction_builder: &_, expression| {
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
                            variant::downcast(
                                context,
                                &instruction_builder,
                                &argument,
                                alternative.type_(),
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
    Ok(pointer::equal(
        instruction_builder.deconstruct_record(argument.clone(), 0)?,
        variant::compile_tag(type_),
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
    let closure_pointer = reference_count::heap::allocate(
        instruction_builder,
        type_::compile_sized_closure(let_.definition(), context.types()),
    )?;

    instruction_builder.store(
        closure::compile_content(
            entry_function::compile(context, let_.definition(), false, variables)?,
            closure::metadata::compile(context, let_.definition())?,
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
                        type_::compile_thunk_payload(let_.definition(), context.types()),
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
                    fmm::types::Pointer::new(type_::compile_unsized_closure(
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

fn compile_synchronize(
    context: &Context,
    instruction_builder: &fmm::build::InstructionBuilder,
    mark: &mir::ir::Synchronize,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let value = compile(context, instruction_builder, mark.expression(), variables)?;

    reference_count::synchronize(instruction_builder, &value, mark.type_(), context.types())?;

    Ok(value)
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
    Ok(if type_::is_record_boxed(record.type_(), context.types()) {
        compile_boxed_record(
            builder,
            allocate_record_heap(context, builder, record.type_())?,
            compile_unboxed_record(context, builder, record, variables)?,
        )?
    } else {
        compile_unboxed_record(context, builder, record, variables)?.into()
    })
}

fn allocate_record_heap(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    record_type: &mir::types::Record,
) -> Result<fmm::build::TypedExpression, CompileError> {
    reference_count::heap::allocate(
        builder,
        type_::compile_unboxed_record(record_type, context.types()),
    )
}

fn compile_boxed_record(
    builder: &fmm::build::InstructionBuilder,
    pointer: impl Into<fmm::build::TypedExpression>,
    unboxed_record: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let unboxed_record = unboxed_record.into();
    let pointer = pointer.into();

    builder.store(
        unboxed_record.clone(),
        fmm::build::bit_cast(
            fmm::types::Pointer::new(unboxed_record.type_().clone()),
            pointer.clone(),
        ),
    );

    Ok(fmm::build::bit_cast(type_::compile_boxed_record(), pointer).into())
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
                        variant::downcast(
                            context,
                            &instruction_builder,
                            &operand,
                            operation.type_(),
                        )?,
                    )])
                    .collect(),
            )?))
        },
        |instruction_builder| Ok(instruction_builder.branch(operand.clone())),
    )
}
