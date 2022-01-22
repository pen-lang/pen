use crate::infra::FilePath;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub fn calculate(source_file: &FilePath) -> String {
    let mut hasher = DefaultHasher::new();

    source_file.hash(&mut hasher);

    format!(
        "{}_{:x}",
        source_file.file_name().with_extension(""),
        hasher.finish()
    )
}
