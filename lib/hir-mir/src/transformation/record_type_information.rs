use hir::types;

pub fn compile_equal_function_name(record_type: &types::Record) -> String {
    format!("{}.$equal", record_type.name())
}

pub fn compile_hash_function_name(record_type: &types::Record) -> String {
    format!("{}.$hash", record_type.name())
}
