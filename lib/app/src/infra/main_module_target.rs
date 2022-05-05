use super::ModuleTargetSource;
use crate::infra::FilePath;
use std::collections::BTreeMap;

pub struct MainModuleTarget {
    source_file: FilePath,
    object_file: FilePath,
    context_interface_files: BTreeMap<String, FilePath>,
    source: ModuleTargetSource,
}

impl MainModuleTarget {
    pub fn new(
        source_file: FilePath,
        object_file: FilePath,
        context_interface_files: BTreeMap<String, FilePath>,
        source: ModuleTargetSource,
    ) -> Self {
        Self {
            source_file,
            object_file,
            context_interface_files,
            source,
        }
    }

    pub fn source_file(&self) -> &FilePath {
        &self.source_file
    }

    pub fn object_file(&self) -> &FilePath {
        &self.object_file
    }

    pub fn context_interface_files(&self) -> &BTreeMap<String, FilePath> {
        &self.context_interface_files
    }

    pub fn source(&self) -> &ModuleTargetSource {
        &self.source
    }
}
