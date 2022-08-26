use super::error::CompileError;
use crate::{
    call, closure, context::Context, entry_function, pointer, record, reference_count, type_,
    variant,
};
use fnv::FnvHashMap;

pub fn compile(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    expression: &mir::ir::Expression,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let compile = |expression, variables: &_| compile(context, builder, expression, variables);

    Ok(match expression {
        mir::ir::Expression::ArithmeticOperation(operation) => {
            compile_arithmetic_operation(context, builder, operation, variables)?.into()
        }
        mir::ir::Expression::Boolean(boolean) => fmm::ir::Primitive::Boolean(*boolean).into(),
        mir::ir::Expression::Case(case) => compile_case(context, builder, case, variables)?,
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
                                    builder,
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
            compile_comparison_operation(context, builder, operation, variables)?.into()
        }
        mir::ir::Expression::DropVariables(drop) => {
            for (variable, type_) in drop.variables() {
                reference_count::drop(builder, &variables[variable], type_, context.types())?;
            }

            compile(drop.expression(), variables)?
        }
        mir::ir::Expression::Call(call) => call::compile(
            builder,
            &compile(call.function(), variables)?,
            &call
                .arguments()
                .iter()
                .map(|argument| compile(argument, variables))
                .collect::<Result<Vec<_>, _>>()?,
        )?,
        mir::ir::Expression::If(if_) => compile_if(context, builder, if_, variables)?,
        mir::ir::Expression::Let(let_) => compile_let(context, builder, let_, variables)?,
        mir::ir::Expression::LetRecursive(let_) => {
            compile_let_recursive(context, builder, let_, variables)?
        }
        mir::ir::Expression::Synchronize(synchronize) => {
            compile_synchronize(context, builder, synchronize, variables)?
        }
        mir::ir::Expression::None => fmm::ir::Undefined::new(type_::compile_none()).into(),
        mir::ir::Expression::Number(number) => fmm::ir::Primitive::Float64(*number).into(),
        mir::ir::Expression::Record(record) => compile_record(context, builder, record, variables)?,
        mir::ir::Expression::RecordField(field) => {
            let record_type = field.type_().clone();
            let field_index = field.index();

            let record = compile(field.record(), variables)?;
            let field = record::get_field(context, builder, &record, field.type_(), field.index())?;

            let field = reference_count::clone(
                builder,
                &field,
                &context.types()[record_type.name()].fields()[field_index],
                context.types(),
            )?;
            reference_count::drop(builder, &record, &record_type.into(), context.types())?;

            field
        }
        mir::ir::Expression::RecordUpdate(update) => {
            let record = compile(update.record(), variables)?;
            let fields = update
                .fields()
                .iter()
                .map(|field| Ok((field.index(), compile(field.expression(), variables)?)))
                .collect::<Result<FnvHashMap<_, _>, CompileError>>()?;

            let compile_unboxed = |builder: &fmm::build::InstructionBuilder,
                                   cloned: bool|
             -> Result<_, CompileError> {
                let record_fields = context.types()[update.type_().name()].fields();
                let pointer = builder.allocate_stack(type_::compile_unboxed_record(
                    update.type_(),
                    context.types(),
                ));

                builder.store(
                    if type_::is_record_boxed(update.type_(), context.types()) {
                        builder.load(fmm::build::bit_cast(
                            pointer.type_().clone(),
                            record.clone(),
                        ))?
                    } else {
                        record.clone()
                    },
                    pointer.clone(),
                );

                if cloned {
                    for (index, field_type) in record_fields.iter().enumerate() {
                        if fields.contains_key(&index) {
                            builder.store(
                                fmm::ir::Undefined::new(type_::compile(
                                    field_type,
                                    context.types(),
                                )),
                                fmm::build::record_address(pointer.clone(), index)?,
                            );
                        }
                    }

                    builder.store(
                        reference_count::record::clone_unboxed(
                            context,
                            builder,
                            &builder.load(pointer.clone())?,
                            update.type_(),
                        )?,
                        pointer.clone(),
                    )
                }

                for (index, field_type) in record_fields.iter().enumerate() {
                    if let Some(expression) = fields.get(&index) {
                        if !cloned {
                            reference_count::drop(
                                builder,
                                &record::get_field(
                                    context,
                                    builder,
                                    &record,
                                    update.type_(),
                                    index,
                                )?,
                                field_type,
                                context.types(),
                            )?;
                        }

                        builder.store(
                            expression.clone(),
                            fmm::build::record_address(pointer.clone(), index)?,
                        );
                    }
                }

                Ok(builder.load(pointer)?)
            };

            if type_::is_record_boxed(update.type_(), context.types()) {
                builder.if_(
                    reference_count::pointer::is_unique(builder, &record)?,
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
                compile_unboxed(builder, false)?.into()
            }
        }
        mir::ir::Expression::ByteString(string) => {
            if string.value().is_empty() {
                fmm::ir::Undefined::new(type_::compile_string()).into()
            } else {
                fmm::build::bit_cast(
                    type_::compile_string(),
                    fmm::build::record_address(
                        context.module_builder().define_anonymous_variable(
                            reference_count::block::compile_static(fmm::build::record(
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
                            ))?,
                            fmm::ir::VariableDefinitionOptions::new()
                                .set_address_named(false)
                                .set_linkage(fmm::ir::Linkage::Internal)
                                .set_mutable(false),
                        ),
                        1,
                    )?,
                )
                .into()
            }
        }
        mir::ir::Expression::TryOperation(operation) => {
            compile_try_operation(context, builder, operation, variables)?
        }
        mir::ir::Expression::Variable(variable) => variables[variable.name()].clone(),
        mir::ir::Expression::Variant(variant) => variant::upcast(
            context,
            builder,
            &compile(variant.payload(), variables)?,
            variant.type_(),
        )?,
    })
}

fn compile_if(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    if_: &mir::ir::If,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let compile = |builder: &_, expression| compile(context, builder, expression, variables);

    builder.if_(
        compile(builder, if_.condition())?,
        |builder| Ok(builder.branch(compile(&builder, if_.then())?)),
        |builder| Ok(builder.branch(compile(&builder, if_.else_())?)),
    )
}

fn compile_case(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    case: &mir::ir::Case,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(compile_alternatives(
        context,
        builder,
        compile(context, builder, case.argument(), variables)?,
        case.alternatives(),
        case.default_alternative(),
        variables,
    )?
    .unwrap())
}

fn compile_alternatives(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
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
                    builder,
                    alternative.expression(),
                    &variables
                        .clone()
                        .into_iter()
                        .chain([(alternative.name().into(), argument)])
                        .collect(),
                )
            })
            .transpose()?,
        [alternative, ..] => Some(builder.if_(
            alternative.types().iter().fold(
                Ok(fmm::build::TypedExpression::from(
                    fmm::ir::Primitive::Boolean(false),
                )),
                |result, type_| -> Result<_, CompileError> {
                    Ok(fmm::build::bitwise_operation(
                        fmm::ir::BitwiseOperator::Or,
                        result?,
                        compile_tag_comparison(builder, &argument, type_)?,
                    )?
                    .into())
                },
            )?,
            |builder| -> Result<_, CompileError> {
                Ok(builder.branch(compile(
                    context,
                    &builder,
                    alternative.expression(),
                    &variables
                        .clone()
                        .into_iter()
                        .chain([(
                            alternative.name().into(),
                            variant::downcast(context, &builder, &argument, alternative.type_())?,
                        )])
                        .collect(),
                )?))
            },
            |builder| {
                Ok(
                    if let Some(expression) = compile_alternatives(
                        context,
                        &builder,
                        argument.clone(),
                        &alternatives[1..],
                        default_alternative,
                        variables,
                    )? {
                        builder.branch(expression)
                    } else {
                        builder.unreachable()
                    },
                )
            },
        )?),
    })
}

fn compile_tag_comparison(
    builder: &fmm::build::InstructionBuilder,
    argument: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(pointer::equal(
        builder.deconstruct_record(argument.clone(), 0)?,
        variant::compile_tag(type_),
    )?)
}

fn compile_let(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    let_: &mir::ir::Let,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let compile = |expression, variables| compile(context, builder, expression, variables);

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
    builder: &fmm::build::InstructionBuilder,
    let_: &mir::ir::LetRecursive,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let closure_pointer = reference_count::heap::allocate(
        builder,
        type_::compile_sized_closure(let_.definition(), context.types()),
    )?;

    builder.store(
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
        builder,
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
    builder: &fmm::build::InstructionBuilder,
    synchronize: &mir::ir::Synchronize,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let value = compile(context, builder, synchronize.expression(), variables)?;

    reference_count::synchronize(builder, &value, synchronize.type_(), context.types())?;

    Ok(value)
}

fn compile_arithmetic_operation(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    operation: &mir::ir::ArithmeticOperation,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::ir::ArithmeticOperation, CompileError> {
    let compile = |expression| compile(context, builder, expression, variables);

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
    builder: &fmm::build::InstructionBuilder,
    operation: &mir::ir::ComparisonOperation,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::ir::ComparisonOperation, CompileError> {
    let compile = |expression| compile(context, builder, expression, variables);

    let lhs = compile(operation.lhs())?;
    let rhs = compile(operation.rhs())?;

    Ok(fmm::build::comparison_operation(
        match operation.operator() {
            mir::ir::ComparisonOperator::Equal => fmm::ir::ComparisonOperator::Equal,
            mir::ir::ComparisonOperator::NotEqual => fmm::ir::ComparisonOperator::NotEqual,
            mir::ir::ComparisonOperator::GreaterThan => {
                fmm::ir::ComparisonOperator::GreaterThan(false)
            }
            mir::ir::ComparisonOperator::GreaterThanOrEqual => {
                fmm::ir::ComparisonOperator::GreaterThanOrEqual(false)
            }
            mir::ir::ComparisonOperator::LessThan => fmm::ir::ComparisonOperator::LessThan(false),
            mir::ir::ComparisonOperator::LessThanOrEqual => {
                fmm::ir::ComparisonOperator::LessThanOrEqual(false)
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
    builder: &fmm::build::InstructionBuilder,
    operation: &mir::ir::TryOperation,
    variables: &FnvHashMap<String, fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let operand = compile(context, builder, operation.operand(), variables)?;

    builder.if_(
        compile_tag_comparison(builder, &operand, operation.type_())?,
        |builder| -> Result<_, CompileError> {
            Ok(builder.return_(compile(
                context,
                &builder,
                operation.then(),
                &variables
                    .clone()
                    .into_iter()
                    .chain([(
                        operation.name().into(),
                        variant::downcast(context, &builder, &operand, operation.type_())?,
                    )])
                    .collect(),
            )?))
        },
        |builder| Ok(builder.branch(operand.clone())),
    )
}
