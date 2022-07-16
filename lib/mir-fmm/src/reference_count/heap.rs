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
