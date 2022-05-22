use super::super::type_;
use fnv::FnvHashMap;

pub fn get_record_clone_function_name(name: &str) -> String {
    format!("mir_clone_{}", name)
}

pub fn get_record_drop_function_name(name: &str) -> String {
    format!("mir_drop_{}", name)
}

pub fn get_record_drop_or_reuse_function_name(name: &str) -> String {
    format!("mir_drop_or_reuse_{}", name)
}

pub fn compile_record_clone_function_type(
    record: &mir::types::Record,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> fmm::types::Function {
    let record = type_::compile_record(record, types);

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
        vec![type_::compile_record(record, types)],
        fmm::types::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
    )
}

pub fn compile_record_drop_or_reuse_function_type(
    record: &mir::types::Record,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> fmm::types::Function {
    let record_type = type_::compile_record(record, types);

    fmm::types::Function::new(
        vec![record_type.clone()],
        record_type,
        fmm::types::CallingConvention::Target,
    )
}
