use super::super::types;
use std::collections::BTreeMap;

pub fn get_record_clone_function_name(name: &str) -> String {
    format!("mir_clone_{}", name)
}

pub fn get_record_drop_function_name(name: &str) -> String {
    format!("mir_drop_{}", name)
}

pub fn compile_record_rc_function_type(
    record: &mir::types::Record,
    types: &BTreeMap<String, mir::types::RecordBody>,
) -> fmm::types::Function {
    fmm::types::Function::new(
        vec![types::compile_record(record, types)],
        fmm::types::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
    )
}
