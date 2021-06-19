use crate::infra::FilePath;

pub struct ModuleTarget {
    package_prefix: String,
    module_prefix: String,
    package_directory: FilePath,
    source_file_path: FilePath,
    object_file_path: FilePath,
    interface_file_path: FilePath,
}

impl ModuleTarget {
    pub fn new(
        package_prefix: impl Into<String>,
        module_prefix: impl Into<String>,
        package_directory: FilePath,
        source_file_path: FilePath,
        object_file_path: FilePath,
        interface_file_path: FilePath,
    ) -> Self {
        Self {
            package_prefix: package_prefix.into(),
            module_prefix: module_prefix.into(),
            package_directory,
            source_file_path,
            object_file_path,
            interface_file_path,
        }
    }

    pub fn package_prefix(&self) -> &str {
        &self.package_prefix
    }

    pub fn module_prefix(&self) -> &str {
        &self.module_prefix
    }

    pub fn package_directory(&self) -> &FilePath {
        &self.package_directory
    }

    pub fn source_file_path(&self) -> &FilePath {
        &self.source_file_path
    }

    pub fn object_file_path(&self) -> &FilePath {
        &self.object_file_path
    }

    pub fn interface_file_path(&self) -> &FilePath {
        &self.interface_file_path
    }
}
