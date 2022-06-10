use super::{super::error::CompileError, count, heap};

// Reference counts are negative for synchronized memory blocks and otherwise
// positive. References to static memory blocks are tagged.

pub fn clone(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    builder.if_(
        is_heap(pointer)?,
        |builder| {
            let count_pointer = heap::get_count_pointer(pointer)?;
            let count =
                builder.atomic_load(count_pointer.clone(), fmm::ir::AtomicOrdering::Relaxed)?;

            builder.if_(
                count::is_synchronized(&count)?,
                |builder| -> Result<_, CompileError> {
                    builder.atomic_operation(
                        fmm::ir::AtomicOperator::Subtract,
                        count_pointer.clone(),
                        count::compile(1),
                        fmm::ir::AtomicOrdering::Relaxed,
                    )?;

                    Ok(builder.branch(fmm::ir::VOID_VALUE.clone()))
                },
                |builder| {
                    builder.store(
                        fmm::build::arithmetic_operation(
                            fmm::ir::ArithmeticOperator::Add,
                            count.clone(),
                            count::compile(1),
                        )?,
                        count_pointer.clone(),
                    );

                    Ok(builder.branch(fmm::ir::VOID_VALUE.clone()))
                },
            )?;

            Ok(builder.branch(pointer.clone()))
        },
        |builder| Ok(builder.branch(pointer.clone())),
    )
}

pub fn drop(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
    drop_content: impl Fn(&fmm::build::InstructionBuilder) -> Result<(), CompileError>,
) -> Result<(), CompileError> {
    let drop_inner = |builder: &_| {
        drop_content(builder)?;
        heap::free(builder, pointer.clone())
    };

    builder.if_(
        is_heap(pointer)?,
        |builder| -> Result<_, CompileError> {
            let count_pointer = heap::get_count_pointer(pointer)?;
            let count =
                builder.atomic_load(count_pointer.clone(), fmm::ir::AtomicOrdering::Relaxed)?;

            Ok(builder.branch(builder.if_(
                count::is_synchronized(&count)?,
                |builder| -> Result<_, CompileError> {
                    Ok(builder.branch(builder.if_(
                        count::is_initial(&builder.atomic_operation(
                            fmm::ir::AtomicOperator::Add,
                            count_pointer.clone(),
                            count::compile(1),
                            fmm::ir::AtomicOrdering::Release,
                        )?)?,
                        |builder| -> Result<_, CompileError> {
                            builder.fence(fmm::ir::AtomicOrdering::Acquire);
                            drop_inner(&builder)?;

                            Ok(builder.branch(fmm::ir::VOID_VALUE.clone()))
                        },
                        |builder| Ok(builder.branch(fmm::ir::VOID_VALUE.clone())),
                    )?))
                },
                |builder| {
                    Ok(builder.branch(builder.if_(
                        count::is_initial(&count)?,
                        |builder| -> Result<_, CompileError> {
                            drop_inner(&builder)?;

                            Ok(builder.branch(fmm::ir::VOID_VALUE.clone()))
                        },
                        |builder| {
                            builder.store(
                                fmm::build::arithmetic_operation(
                                    fmm::ir::ArithmeticOperator::Subtract,
                                    count.clone(),
                                    count::compile(1),
                                )?,
                                count_pointer.clone(),
                            );

                            Ok(builder.branch(fmm::ir::VOID_VALUE.clone()))
                        },
                    )?))
                },
            )?))
        },
        |builder| Ok(builder.branch(fmm::ir::VOID_VALUE.clone())),
    )?;

    Ok(())
}

pub fn is_owned(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    builder.if_(
        is_heap(pointer)?,
        |builder| {
            Ok(builder.branch(count::is_initial(&builder.atomic_load(
                heap::get_count_pointer(pointer)?,
                fmm::ir::AtomicOrdering::Relaxed,
            )?)?))
        },
        |builder| Ok(builder.branch(fmm::ir::Primitive::Boolean(false))),
    )
}

pub fn synchronize(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
) -> Result<(), CompileError> {
    builder.if_(
        is_heap(pointer)?,
        |builder| -> Result<_, CompileError> {
            let pointer = heap::get_count_pointer(pointer)?;

            builder.if_(
                count::is_synchronized(
                    &builder.atomic_load(pointer.clone(), fmm::ir::AtomicOrdering::Relaxed)?,
                )?,
                |builder| -> Result<_, CompileError> {
                    Ok(builder.branch(fmm::ir::VOID_VALUE.clone()))
                },
                |builder| {
                    builder.atomic_store(
                        count::synchronize(
                            &builder
                                .atomic_load(pointer.clone(), fmm::ir::AtomicOrdering::Relaxed)?,
                        )?,
                        pointer.clone(),
                        fmm::ir::AtomicOrdering::Relaxed,
                    );

                    Ok(builder.branch(fmm::ir::VOID_VALUE.clone()))
                },
            )?;

            Ok(builder.branch(fmm::ir::VOID_VALUE.clone()))
        },
        |builder| Ok(builder.branch(fmm::ir::VOID_VALUE.clone())),
    )?;

    Ok(())
}

pub fn tag_as_static(
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::bit_cast(
        pointer.type_().clone(),
        fmm::build::bitwise_operation(
            fmm::ir::BitwiseOperator::Or,
            fmm::build::bit_cast(fmm::types::Primitive::PointerInteger, pointer.clone()),
            fmm::ir::Primitive::PointerInteger(1),
        )?,
    )
    .into())
}

pub fn untag(
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::bit_cast(
        pointer.type_().clone(),
        fmm::build::bitwise_operation(
            fmm::ir::BitwiseOperator::And,
            fmm::build::bit_cast(fmm::types::Primitive::PointerInteger, pointer.clone()),
            fmm::build::bitwise_not_operation(fmm::ir::Primitive::PointerInteger(1))?,
        )?,
    )
    .into())
}

fn is_heap(
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::bitwise_operation(
        fmm::ir::BitwiseOperator::And,
        fmm::build::comparison_operation(
            fmm::ir::ComparisonOperator::NotEqual,
            fmm::build::bit_cast(fmm::types::Primitive::PointerInteger, pointer.clone()),
            fmm::ir::Undefined::new(fmm::types::Primitive::PointerInteger),
        )?,
        fmm::build::comparison_operation(
            fmm::ir::ComparisonOperator::NotEqual,
            fmm::build::bitwise_operation(
                fmm::ir::BitwiseOperator::And,
                fmm::build::bit_cast(fmm::types::Primitive::PointerInteger, pointer.clone()),
                fmm::ir::Primitive::PointerInteger(1),
            )?,
            fmm::ir::Primitive::PointerInteger(1),
        )?,
    )?
    .into())
}
