use crate::infra::FilePath;

pub struct MainModuleTarget {
    source_file: FilePath,
    object_file: FilePath,
    main_function_interface_file: FilePath,
}

impl MainModuleTarget {
    pub fn new(
        source_file: FilePath,
        object_file: FilePath,
        context_interface_file: FilePath,
    ) -> Self {
        Self {
            source_file,
            object_file,
            main_function_interface_file: context_interface_file,
        }
    }

    pub fn source_file(&self) -> &FilePath {
        &self.source_file
    }

    pub fn object_file(&self) -> &FilePath {
        &self.object_file
    }

    pub fn context_interface_file(&self) -> &FilePath {
        &self.main_function_interface_file
    }
}
