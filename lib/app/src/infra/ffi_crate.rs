use crate::infra::FilePath;

pub struct FfiCrate {
    directory: FilePath,
    library_name: String,
}

impl FfiCrate {
    pub fn new(directory: FilePath, library_name: String) -> Self {
        Self {
            directory,
            library_name,
        }
    }

    pub fn directory(&self) -> &FilePath {
        &self.directory
    }

    pub fn library_name(&self) -> &str {
        &self.library_name
    }
}
