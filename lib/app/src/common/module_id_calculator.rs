use crate::infra::FilePath;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn calculate_module_id(source_file: &FilePath) -> String {
    let mut hasher = DefaultHasher::new();

    source_file.hash(&mut hasher);

    format!("{:x}", hasher.finish())
}
