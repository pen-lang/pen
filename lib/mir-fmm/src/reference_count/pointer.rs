use super::{super::error::CompileError, count, heap};

// Reference counts are negative for synchronized memory blocks and otherwise positive.
// References to static memory blocks are tagged.

pub fn clone(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    if_heap_pointer(
        builder,
        pointer,
        |builder| {
            let count_pointer = heap::get_counter_pointer(pointer)?;
            let count =
                builder.atomic_load(count_pointer.clone(), fmm::ir::AtomicOrdering::Relaxed)?;

            builder.if_(
                is_count_synchronized(&count)?,
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
    if_heap_pointer(
        builder,
        pointer,
        |builder| {
            let count_pointer = heap::get_counter_pointer(pointer)?;
            let count =
                builder.atomic_load(count_pointer.clone(), fmm::ir::AtomicOrdering::Relaxed)?;

            builder.if_(
                is_count_initial(&builder.if_(
                    is_count_synchronized(&count)?,
                    |builder| -> Result<_, CompileError> {
                        Ok(builder.branch(builder.atomic_operation(
                            fmm::ir::AtomicOperator::Add,
                            count_pointer.clone(),
                            count::compile(1),
                            fmm::ir::AtomicOrdering::Release,
                        )?))
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

                        Ok(builder.branch(count.clone()))
                    },
                )?)?,
                |builder| -> Result<_, CompileError> {
                    builder.fence(fmm::ir::AtomicOrdering::Acquire);

                    drop_content(&builder)?;

                    heap::free(
                        &builder,
                        fmm::build::bit_cast(
                            fmm::types::GENERIC_POINTER_TYPE.clone(),
                            pointer.clone(),
                        ),
                    )?;

                    Ok(builder.branch(fmm::ir::VOID_VALUE.clone()))
                },
                |builder| Ok(builder.branch(fmm::ir::VOID_VALUE.clone())),
            )?;

            Ok(builder.branch(fmm::ir::VOID_VALUE.clone()))
        },
        |builder| Ok(builder.branch(fmm::ir::VOID_VALUE.clone())),
    )?;

    Ok(())
}

pub fn is_owned(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    if_heap_pointer(
        builder,
        pointer,
        |builder| {
            Ok(builder.branch(fmm::build::comparison_operation(
                fmm::ir::ComparisonOperator::Equal,
                builder.atomic_load(
                    heap::get_counter_pointer(pointer)?,
                    fmm::ir::AtomicOrdering::Relaxed,
                )?,
                count::compile_initial(),
            )?))
        },
        |builder| Ok(builder.branch(fmm::ir::Primitive::Boolean(false))),
    )
}

pub fn synchronize(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
) -> Result<(), CompileError> {
    if_heap_pointer(
        builder,
        pointer,
        |builder| {
            let pointer = heap::get_counter_pointer(pointer)?;

            builder.if_(
                fmm::build::comparison_operation(
                    fmm::ir::ComparisonOperator::LessThan,
                    builder.atomic_load(pointer.clone(), fmm::ir::AtomicOrdering::Relaxed)?,
                    count::compile_initial(),
                )?,
                |builder| Ok::<_, CompileError>(builder.branch(fmm::ir::VOID_VALUE.clone())),
                |builder| {
                    builder.atomic_store(
                        fmm::build::arithmetic_operation(
                            fmm::ir::ArithmeticOperator::Subtract,
                            count::compile_initial(),
                            builder
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

pub fn is_synchronized(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    if_heap_pointer(
        builder,
        pointer,
        |builder| {
            Ok(builder.branch(is_count_synchronized(&builder.atomic_load(
                heap::get_counter_pointer(pointer)?,
                fmm::ir::AtomicOrdering::Relaxed,
            )?)?))
        },
        |builder| Ok(builder.branch(fmm::ir::Primitive::Boolean(true))),
    )
}

fn is_count_synchronized(
    count: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::comparison_operation(
        fmm::ir::ComparisonOperator::LessThan,
        count.clone(),
        count::compile_initial(),
    )?
    .into())
}

fn is_count_initial(
    count: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::comparison_operation(
        fmm::ir::ComparisonOperator::Equal,
        count.clone(),
        count::compile_initial(),
    )?
    .into())
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

fn if_heap_pointer(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
    then: impl Fn(fmm::build::InstructionBuilder) -> Result<fmm::ir::Block, CompileError>,
    else_: impl Fn(fmm::build::InstructionBuilder) -> Result<fmm::ir::Block, CompileError>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let then = &then;
    let else_ = &else_;

    builder.if_(
        fmm::build::comparison_operation(
            fmm::ir::ComparisonOperator::NotEqual,
            fmm::build::bit_cast(fmm::types::Primitive::PointerInteger, pointer.clone()),
            fmm::ir::Undefined::new(fmm::types::Primitive::PointerInteger),
        )?,
        |builder| Ok(builder.branch(builder.if_(is_heap_pointer(pointer)?, then, else_)?)),
        else_,
    )
}

fn is_heap_pointer(
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::comparison_operation(
        fmm::ir::ComparisonOperator::NotEqual,
        fmm::build::bitwise_operation(
            fmm::ir::BitwiseOperator::And,
            fmm::build::bit_cast(fmm::types::Primitive::PointerInteger, pointer.clone()),
            fmm::ir::Primitive::PointerInteger(1),
        )?,
        fmm::ir::Primitive::PointerInteger(1),
    )?
    .into())
}
