use crate::infra::{
    BuildScriptDependencyCompiler, FilePathConfiguration, FilePathDisplayer, FileSystem,
    PackageConfigurationReader,
};
use std::sync::Arc;

pub struct ModuleDependencyResolverInfrastructure {
    pub build_script_dependency_compiler: Arc<dyn BuildScriptDependencyCompiler>,
    pub package_configuration_reader: Arc<dyn PackageConfigurationReader>,
    pub file_system: Arc<dyn FileSystem>,
    pub file_path_displayer: Arc<dyn FilePathDisplayer>,
    pub file_path_configuration: Arc<FilePathConfiguration>,
}
