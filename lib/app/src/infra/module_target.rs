use super::ModuleTargetSource;
use crate::infra::FilePath;

pub struct ModuleTarget {
    package_directory: FilePath,
    source_file: FilePath,
    object_file: FilePath,
    interface_file: FilePath,
    source: ModuleTargetSource,
}

impl ModuleTarget {
    pub fn new(
        package_directory: FilePath,
        source_file: FilePath,
        object_file: FilePath,
        interface_file: FilePath,
        source: ModuleTargetSource,
    ) -> Self {
        Self {
            package_directory,
            source_file,
            object_file,
            interface_file,
            source,
        }
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

    pub fn source(&self) -> &ModuleTargetSource {
        &self.source
    }
}
