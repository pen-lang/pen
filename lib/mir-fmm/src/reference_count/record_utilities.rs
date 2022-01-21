use super::super::types;
use fnv::FnvHashMap;

pub fn get_record_clone_function_name(name: &str) -> String {
    format!("mir_clone_{}", name)
}

pub fn get_record_drop_function_name(name: &str) -> String {
    format!("mir_drop_{}", name)
}

pub fn compile_record_clone_function_type(
    record: &mir::types::Record,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> fmm::types::Function {
    let record = types::compile_record(record, types);

    fmm::types::Function::new(
        vec![record.clone()],
        record,
        fmm::types::CallingConvention::Target,
    )
}

pub fn compile_record_drop_function_type(
    record: &mir::types::Record,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> fmm::types::Function {
    fmm::types::Function::new(
        vec![types::compile_record(record, types)],
        fmm::types::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
    )
}
