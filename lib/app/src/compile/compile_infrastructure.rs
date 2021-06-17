use crate::infra::{FilePathDisplayer, FileSystem};

pub struct CompileInfrastructure {
    pub file_system: Box<dyn FileSystem>,
    pub file_path_displayer: Box<dyn FilePathDisplayer>,
}
