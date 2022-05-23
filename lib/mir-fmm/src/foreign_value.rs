use crate::{box_, type_, CompileError};
use fnv::FnvHashMap;

pub fn convert_to_foreign(
    builder: &fmm::build::InstructionBuilder,
    value: impl Into<fmm::build::TypedExpression>,
    type_: &mir::types::Type,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let value = value.into();

    Ok(if type_::foreign::should_box_payload(type_, types) {
        box_::box_(builder, value)?
    } else {
        value
    })
}

pub fn convert_from_foreign(
    builder: &fmm::build::InstructionBuilder,
    value: impl Into<fmm::build::TypedExpression>,
    type_: &mir::types::Type,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let value = value.into();

    Ok(if type_::foreign::should_box_payload(type_, types) {
        box_::unbox(builder, value, type_, types)?
    } else {
        value
    })
}
