use crate::{error::CompileError, types};
use std::collections::HashMap;

pub fn get_record_field(
    builder: &fmm::build::InstructionBuilder,
    record: &fmm::build::TypedExpression,
    type_: &mir::types::Record,
    element_index: usize,
    types: &HashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.deconstruct_record(
        if types::is_record_boxed(type_, types) {
            builder.load(fmm::build::bit_cast(
                fmm::types::Pointer::new(types::compile_unboxed_record(type_, types)),
                record.clone(),
            ))?
        } else {
            record.clone()
        },
        element_index,
    )?)
}
