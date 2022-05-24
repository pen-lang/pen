use crate::{type_, CompileError};
use fnv::FnvHashMap;

pub fn compile_payload(
    type_: &mir::types::Type,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::types::Type, CompileError> {
    let fmm_type = type_::compile(type_, types);

    Ok(if should_box_payload(type_, types)? {
        fmm::types::Pointer::new(fmm_type).into()
    } else {
        fmm_type
    })
}

pub fn should_box_payload(
    type_: &mir::types::Type,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<bool, CompileError> {
    Ok(match type_ {
        mir::types::Type::Record(record_type) => {
            type_::is_record_boxed(record_type, types) != is_record_boxed(record_type, types)
        }
        mir::types::Type::Variant => return Err(CompileError::NestedVariant),
        mir::types::Type::Boolean
        | mir::types::Type::ByteString
        | mir::types::Type::Function(_)
        | mir::types::Type::None
        | mir::types::Type::Number => false,
    })
}

fn is_record_boxed(
    record: &mir::types::Record,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> bool {
    let body_type = &types[record.name()];

    body_type.fields().len() > 1
        || body_type.fields().iter().any(|type_| {
            // Variants always take two words.
            matches!(
                type_,
                mir::types::Type::Record(_) | mir::types::Type::Variant
            )
        })
}
