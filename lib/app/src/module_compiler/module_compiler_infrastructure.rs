use crate::infra::{FilePathDisplayer, FileSystem};
use std::sync::Arc;

pub struct ModuleCompilerInfrastructure {
    pub file_system: Arc<dyn FileSystem>,
    pub file_path_displayer: Arc<dyn FilePathDisplayer>,
}
