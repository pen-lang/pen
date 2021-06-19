use crate::{
    common::FilePathConfiguration,
    infra::{DependencyCompiler, FilePathDisplayer, FileSystem},
};
use std::sync::Arc;

pub struct CompileDependencyInfrastructure {
    pub dependency_compiler: Arc<dyn DependencyCompiler>,
    pub file_system: Arc<dyn FileSystem>,
    pub file_path_displayer: Arc<dyn FilePathDisplayer>,
    pub file_path_configuration: Arc<FilePathConfiguration>,
}
