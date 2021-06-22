use crate::infra::{
    DependencyCompiler, FilePathConfiguration, FilePathDisplayer, FileSystem,
    PackageConfigurationReader,
};
use std::sync::Arc;

pub struct ModuleDependencyCompilerInfrastructure {
    pub dependency_compiler: Arc<dyn DependencyCompiler>,
    pub package_configuration_reader: Arc<dyn PackageConfigurationReader>,
    pub file_system: Arc<dyn FileSystem>,
    pub file_path_displayer: Arc<dyn FilePathDisplayer>,
    pub file_path_configuration: Arc<FilePathConfiguration>,
}
