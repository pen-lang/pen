use super::{super::error::CompileError, block, count};

pub fn allocate(
    builder: &fmm::build::InstructionBuilder,
    type_: impl Into<fmm::types::Type>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let type_ = type_.into();
    let pointer = fmm::build::bit_cast(
        fmm::types::Pointer::new(fmm::types::Record::new(vec![
            count::compile_type().into(),
            type_.clone(),
        ])),
        builder.allocate_heap(fmm::build::size_of(fmm::types::Record::new(vec![
            count::compile_type().into(),
            type_,
        ]))),
    );

    builder.store(
        count::compile_unique(),
        fmm::build::record_address(pointer.clone(), 0)?,
    );

    Ok(fmm::build::record_address(pointer, 1)?.into())
}

pub fn allocate_variadic(
    builder: &fmm::build::InstructionBuilder,
    length: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let count_size = fmm::build::size_of(count::compile_type());
    let pointer = builder.allocate_heap(fmm::build::arithmetic_operation(
        fmm::ir::ArithmeticOperator::Add,
        count_size.clone(),
        length,
    )?);

    builder.store(
        count::compile_unique(),
        fmm::build::bit_cast(
            fmm::types::Pointer::new(count::compile_type()),
            pointer.clone(),
        ),
    );

    Ok(fmm::build::pointer_address(pointer, count_size)?.into())
}

pub fn free(
    builder: &fmm::build::InstructionBuilder,
    pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<(), CompileError> {
    builder.free_heap(fmm::build::bit_cast(
        fmm::types::generic_pointer_type(),
        block::compile_count_pointer(&pointer.into())?,
    ));

    Ok(())
}
