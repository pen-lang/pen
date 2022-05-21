use super::{super::error::CompileError, heap};

pub fn clone_pointer(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    if_heap_pointer(
        builder,
        pointer,
        |builder| {
            increment_count(&builder, pointer, fmm::ir::AtomicOrdering::Relaxed)?;

            Ok(builder.branch(pointer.clone()))
        },
        |builder| Ok(builder.branch(pointer.clone())),
    )
}

pub fn drop_pointer(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
    drop_content: impl Fn(&fmm::build::InstructionBuilder) -> Result<(), CompileError>,
) -> Result<(), CompileError> {
    if_heap_pointer(
        builder,
        pointer,
        |builder| {
            Ok(builder.branch(builder.if_(
                decrement_and_compare_count(&builder, pointer)?,
                |builder| -> Result<_, CompileError> {
                    builder.fence(fmm::ir::AtomicOrdering::Acquire);

                    drop_content(&builder)?;

                    heap::free_heap(
                        &builder,
                        fmm::build::bit_cast(
                            fmm::types::GENERIC_POINTER_TYPE.clone(),
                            pointer.clone(),
                        ),
                    )?;

                    Ok(builder.branch(fmm::ir::VOID_VALUE.clone()))
                },
                |builder| Ok(builder.branch(fmm::ir::VOID_VALUE.clone())),
            )?))
        },
        |builder| Ok(builder.branch(fmm::ir::VOID_VALUE.clone())),
    )?;

    Ok(())
}

pub fn drop_or_reuse_pointer(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
    drop_content: impl Fn(&fmm::build::InstructionBuilder) -> Result<(), CompileError>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let null_pointer = fmm::ir::Undefined::new(pointer.type_().clone());

    if_heap_pointer(
        builder,
        pointer,
        |builder| {
            Ok(builder.branch(builder.if_(
                decrement_and_compare_count(&builder, pointer)?,
                |builder| -> Result<_, CompileError> {
                    increment_count(&builder, pointer, fmm::ir::AtomicOrdering::Acquire)?;

                    drop_content(&builder)?;

                    Ok(builder.branch(pointer.clone()))
                },
                |builder| Ok(builder.branch(null_pointer.clone())),
            )?))
        },
        |builder| Ok(builder.branch(null_pointer.clone())),
    )
}

pub fn is_pointer_owned(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    if_heap_pointer(
        builder,
        pointer,
        |builder| {
            Ok(builder.branch(fmm::build::comparison_operation(
                fmm::ir::ComparisonOperator::Equal,
                // TODO Does this actually need to be atomic?
                builder.atomic_load(
                    get_counter_pointer(pointer)?,
                    fmm::ir::AtomicOrdering::Relaxed,
                )?,
                fmm::ir::Primitive::PointerInteger(heap::INITIAL_COUNT as i64),
            )?))
        },
        |builder| Ok(builder.branch(fmm::ir::Primitive::Boolean(false))),
    )
}

fn increment_count(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
    ordering: fmm::ir::AtomicOrdering,
) -> Result<(), CompileError> {
    builder.atomic_operation(
        fmm::ir::AtomicOperator::Add,
        get_counter_pointer(pointer)?,
        fmm::ir::Primitive::PointerInteger(1),
        ordering,
    )?;

    Ok(())
}

fn decrement_and_compare_count(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::comparison_operation(
        fmm::ir::ComparisonOperator::Equal,
        builder.atomic_operation(
            fmm::ir::AtomicOperator::Subtract,
            get_counter_pointer(pointer)?,
            fmm::ir::Primitive::PointerInteger(1),
            fmm::ir::AtomicOrdering::Release,
        )?,
        fmm::ir::Primitive::PointerInteger(heap::INITIAL_COUNT as i64),
    )?
    .into())
}

pub fn compile_tagged_pointer(
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

pub fn compile_untagged_pointer(
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

fn get_counter_pointer(
    heap_pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    Ok(fmm::build::pointer_address(
        fmm::build::bit_cast(
            fmm::types::Pointer::new(heap::COUNT_TYPE),
            heap_pointer.clone(),
        ),
        fmm::ir::Primitive::PointerInteger(-1),
    )?
    .into())
}
