use super::super::error::CompileError;
use once_cell::sync::Lazy;

pub(super) static HEADER_TYPE: Lazy<fmm::types::Record> = Lazy::new(|| {
    fmm::types::Record::new(vec![
        fmm::types::Primitive::Integer32.into(), // count
        fmm::types::Primitive::Integer32.into(), // tag
    ])
});
pub(super) const INITIAL_COUNT: usize = 0;

pub fn allocate(
    builder: &fmm::build::InstructionBuilder,
    type_: impl Into<fmm::types::Type>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let type_ = type_.into();
    let pointer = fmm::build::bit_cast(
        fmm::types::Pointer::new(fmm::types::Record::new(vec![
            HEADER_TYPE.clone().into(),
            type_.clone(),
        ])),
        builder.allocate_heap(fmm::build::size_of(fmm::types::Record::new(vec![
            HEADER_TYPE.clone().into(),
            type_,
        ]))),
    );

    builder.store(
        fmm::ir::Primitive::PointerInteger(INITIAL_COUNT as i64),
        fmm::build::record_address(pointer.clone(), 0)?,
    );

    Ok(fmm::build::record_address(pointer, 1)?.into())
}

pub fn free(
    builder: &fmm::build::InstructionBuilder,
    pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<(), CompileError> {
    builder.free_heap(fmm::build::bit_cast(
        fmm::types::GENERIC_POINTER_TYPE.clone(),
        fmm::build::pointer_address(
            fmm::build::bit_cast(
                fmm::types::Pointer::new(fmm::types::Primitive::PointerInteger),
                pointer,
            ),
            fmm::ir::Primitive::PointerInteger(-1),
        )?,
    ));

    Ok(())
}
