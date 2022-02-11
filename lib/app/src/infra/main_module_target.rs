use crate::infra::FilePath;

pub struct MainModuleTarget {
    source_file: FilePath,
    object_file: FilePath,
    context_interface_files: Vec<FilePath>,
}

impl MainModuleTarget {
    pub fn new(
        source_file: FilePath,
        object_file: FilePath,
        context_interface_files: Vec<FilePath>,
    ) -> Self {
        Self {
            source_file,
            object_file,
            context_interface_files,
        }
    }

    pub fn source_file(&self) -> &FilePath {
        &self.source_file
    }

    pub fn object_file(&self) -> &FilePath {
        &self.object_file
    }

    pub fn context_interface_files(&self) -> &[FilePath] {
        &self.context_interface_files
    }
}
