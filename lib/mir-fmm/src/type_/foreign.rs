use crate::{type_, CompileError};
use fnv::FnvHashMap;

pub fn compile(
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

pub fn compile_function(
    function: &mir::types::Function,
    calling_convention: mir::ir::CallingConvention,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::types::Function, CompileError> {
    Ok(fmm::types::Function::new(
        function
            .arguments()
            .iter()
            .map(|type_| compile(type_, types))
            .collect::<Result<_, _>>()?,
        compile(function.result(), types)?,
        type_::compile_calling_convention(calling_convention),
    ))
}

pub fn should_box_payload(
    type_: &mir::types::Type,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<bool, CompileError> {
    Ok(match type_ {
        mir::types::Type::Record(record_type) => {
            if type_::is_record_boxed(record_type, types) && !is_record_boxed(record_type, types) {
                return Err(CompileError::UnboxedRecord);
            }

            type_::is_record_boxed(record_type, types) != is_record_boxed(record_type, types)
        }
        mir::types::Type::Variant => true,
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
    !types[record.name()].fields().is_empty()
}
