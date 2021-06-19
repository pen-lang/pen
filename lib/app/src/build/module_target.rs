use crate::infra::FilePath;

pub struct ModuleTarget {
    package_prefix: String,
    module_prefix: String,
    package_directory: FilePath,
    source_file: FilePath,
    object_file: FilePath,
    interface_file: FilePath,
}

impl ModuleTarget {
    pub fn new(
        package_prefix: impl Into<String>,
        module_prefix: impl Into<String>,
        package_directory: FilePath,
        source_file: FilePath,
        object_file: FilePath,
        interface_file: FilePath,
    ) -> Self {
        Self {
            package_prefix: package_prefix.into(),
            module_prefix: module_prefix.into(),
            package_directory,
            source_file,
            object_file,
            interface_file,
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

    pub fn source_file(&self) -> &FilePath {
        &self.source_file
    }

    pub fn object_file(&self) -> &FilePath {
        &self.object_file
    }

    pub fn interface_file(&self) -> &FilePath {
        &self.interface_file
    }
}
