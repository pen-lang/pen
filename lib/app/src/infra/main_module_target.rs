use crate::infra::FilePath;

pub struct MainModuleTarget {
    source_file: FilePath,
    object_file: FilePath,
    system_package_directory: FilePath,
}

impl MainModuleTarget {
    pub fn new(
        source_file: FilePath,
        object_file: FilePath,
        system_package_directory: FilePath,
    ) -> Self {
        Self {
            source_file,
            object_file,
            system_package_directory,
        }
    }

    pub fn source_file(&self) -> &FilePath {
        &self.source_file
    }

    pub fn object_file(&self) -> &FilePath {
        &self.object_file
    }

    pub fn system_package_directory(&self) -> &FilePath {
        &self.system_package_directory
    }
}
