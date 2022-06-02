use super::{super::error::CompileError, count};
use once_cell::sync::Lazy;

const TAG_TYPE: fmm::types::Primitive = fmm::types::Primitive::Integer32;
const EMPTY_TAG: fmm::ir::Primitive = fmm::ir::Primitive::Integer32(0);

static HEADER_TYPE: Lazy<fmm::types::Record> =
    Lazy::new(|| fmm::types::Record::new(vec![count::compile_type().into(), TAG_TYPE.into()]));

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
        fmm::build::record(vec![count::compile_initial().into(), EMPTY_TAG.into()]),
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
            fmm::build::bit_cast(fmm::types::Pointer::new(HEADER_TYPE.clone()), pointer),
            fmm::ir::Primitive::PointerInteger(-1),
        )?,
    ));

    Ok(())
}

pub fn get_counter_pointer(
    heap_pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    Ok(fmm::build::record_address(
        fmm::build::pointer_address(
            fmm::build::bit_cast(
                fmm::types::Pointer::new(HEADER_TYPE.clone()),
                heap_pointer.clone(),
            ),
            fmm::ir::Primitive::PointerInteger(-1),
        )?,
        0,
    )?
    .into())
}
