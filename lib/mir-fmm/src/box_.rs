use crate::{reference_count, type_, CompileError};
use fnv::FnvHashMap;

pub fn box_(
    builder: &fmm::build::InstructionBuilder,
    unboxed: fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let pointer = reference_count::heap::allocate(builder, unboxed.type_().clone())?;

    builder.store(unboxed, pointer.clone());

    Ok(pointer)
}

pub fn unbox(
    builder: &fmm::build::InstructionBuilder,
    pointer: fmm::build::TypedExpression,
    type_: &mir::types::Type,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let loaded = builder.load(fmm::build::bit_cast(
        fmm::types::Pointer::new(type_::compile(type_, types)),
        pointer.clone(),
    ))?;
    let unboxed = reference_count::clone(builder, &loaded, type_, types)?;

    reference_count::pointer::drop(builder, &pointer, |builder| {
        reference_count::drop(builder, &loaded, type_, types)
    })?;

    Ok(unboxed)
}
