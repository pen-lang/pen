use crate::{error::CompileError, type_};
use fnv::FnvHashMap;

pub fn get_record_field(
    builder: &fmm::build::InstructionBuilder,
    record: &fmm::build::TypedExpression,
    type_: &mir::types::Record,
    field_index: usize,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.deconstruct_record(
        if type_::is_record_boxed(type_, types) {
            builder.load(fmm::build::bit_cast(
                fmm::types::Pointer::new(type_::compile_unboxed_record(type_, types)),
                record.clone(),
            ))?
        } else {
            record.clone()
        },
        field_index,
    )?)
}
