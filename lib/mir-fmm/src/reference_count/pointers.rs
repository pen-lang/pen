use super::{super::error::CompileError, heap};

pub fn clone_pointer(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    if_heap_pointer(builder, pointer, |builder| {
        increment_count(builder, pointer, fmm::ir::AtomicOrdering::Relaxed)
    })?;

    Ok(pointer.clone())
}

pub fn drop_pointer(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
    drop_content: impl Fn(&fmm::build::InstructionBuilder) -> Result<(), CompileError>,
) -> Result<(), CompileError> {
    if_heap_pointer(builder, pointer, |builder| {
        builder.if_(
            decrement_and_compare_count(builder, pointer)?,
            |builder| -> Result<_, CompileError> {
                builder.fence(fmm::ir::AtomicOrdering::Acquire);

                drop_content(&builder)?;

                heap::free_heap(
                    &builder,
                    fmm::build::bit_cast(fmm::types::GENERIC_POINTER_TYPE.clone(), pointer.clone()),
                )?;

                Ok(builder.branch(fmm::ir::VOID_VALUE.clone()))
            },
            |builder| Ok(builder.branch(fmm::ir::VOID_VALUE.clone())),
        )?;

        Ok(())
    })?;

    Ok(())
}

pub fn drop_or_reuse_pointer(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
    drop_content: impl Fn(&fmm::build::InstructionBuilder) -> Result<(), CompileError>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let result_pointer = builder.allocate_stack(pointer.type_().clone());

    builder.store(
        fmm::ir::Undefined::new(pointer.type_().clone()),
        result_pointer.clone(),
    );

    if_heap_pointer(builder, pointer, |builder| {
        builder.if_(
            decrement_and_compare_count(builder, pointer)?,
            |builder| -> Result<_, CompileError> {
                increment_count(&builder, pointer, fmm::ir::AtomicOrdering::Acquire)?;

                drop_content(&builder)?;

                builder.store(pointer.clone(), result_pointer.clone());

                Ok(builder.branch(fmm::ir::VOID_VALUE.clone()))
            },
            |builder| Ok(builder.branch(fmm::ir::VOID_VALUE.clone())),
        )?;

        Ok(())
    })?;

    Ok(builder.load(result_pointer)?)
}

pub fn is_pointer_owned(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let result_pointer = builder.allocate_stack(fmm::types::Primitive::Boolean);

    builder.store(fmm::ir::Primitive::Boolean(false), result_pointer.clone());

    if_heap_pointer(builder, pointer, |builder| {
        builder.if_(
            fmm::build::comparison_operation(
                fmm::ir::ComparisonOperator::Equal,
                builder.atomic_load(
                    get_counter_pointer(pointer)?,
                    fmm::ir::AtomicOrdering::Relaxed,
                )?,
                fmm::ir::Primitive::PointerInteger(heap::INITIAL_COUNT as i64),
            )?,
            |builder| -> Result<_, CompileError> {
                builder.store(fmm::ir::Primitive::Boolean(true), result_pointer.clone());

                Ok(builder.branch(fmm::ir::VOID_VALUE.clone()))
            },
            |builder| Ok(builder.branch(fmm::ir::VOID_VALUE.clone())),
        )?;

        Ok(())
    })?;

    Ok(builder.load(result_pointer)?)
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
    then: impl Fn(&fmm::build::InstructionBuilder) -> Result<(), CompileError>,
) -> Result<(), CompileError> {
    builder.if_(
        fmm::build::comparison_operation(
            fmm::ir::ComparisonOperator::NotEqual,
            fmm::build::bit_cast(fmm::types::Primitive::PointerInteger, pointer.clone()),
            fmm::ir::Undefined::new(fmm::types::Primitive::PointerInteger),
        )?,
        |builder| -> Result<_, CompileError> {
            builder.if_(
                is_heap_pointer(pointer)?,
                |builder| -> Result<_, CompileError> {
                    then(&builder)?;
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
