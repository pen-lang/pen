use crate::infra::FilePath;

pub struct ModuleBuildTarget {
    module_prefix: String,
    source_file_path: FilePath,
    target_file_path: FilePath,
}

impl ModuleBuildTarget {
    pub fn new(
        module_prefix: impl Into<String>,
        source_file_path: FilePath,
        target_file_path: FilePath,
    ) -> Self {
        Self {
            module_prefix: module_prefix.into(),
            source_file_path,
            target_file_path,
        }
    }

    pub fn module_prefix(&self) -> &str {
        &self.module_prefix
    }

    pub fn source_file_path(&self) -> &FilePath {
        &self.source_file_path
    }

    pub fn target_file_path(&self) -> &FilePath {
        &self.target_file_path
    }
}
