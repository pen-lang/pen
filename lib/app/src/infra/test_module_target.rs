use super::ModuleTargetSource;
use crate::infra::FilePath;

pub struct TestModuleTarget {
    package_directory: FilePath,
    source_file: FilePath,
    object_file: FilePath,
    test_information_file: FilePath,
    source: ModuleTargetSource,
}

impl TestModuleTarget {
    pub fn new(
        package_directory: FilePath,
        source_file: FilePath,
        object_file: FilePath,
        test_information_file: FilePath,
        source: ModuleTargetSource,
    ) -> Self {
        Self {
            package_directory,
            source_file,
            object_file,
            test_information_file,
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

    pub fn test_information_file(&self) -> &FilePath {
        &self.test_information_file
    }

    pub fn source(&self) -> &ModuleTargetSource {
        &self.source
    }
}
