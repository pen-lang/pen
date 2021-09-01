
use crate::types;

pub fn compile_equal_function_name(record_type: &types::Record) -> String {
    format!("{}.$equal", record_type.name())
}
