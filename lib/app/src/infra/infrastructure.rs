use super::{
    BuildScriptDependencyCompiler, ExternalPackageInitializer, FilePathConfiguration,
    FilePathDisplayer, FileSystem, BuildScriptCompiler, ModuleBuilder,
    PackageConfigurationReader,
};
use std::sync::Arc;

// Infrastructure is agnostic about the following concepts and their information
// should be passed through function arguments instead.
//
// - Output directory
// - Main package directory
pub struct Infrastructure {
    pub build_script_dependency_compiler: Arc<dyn BuildScriptDependencyCompiler>,
    pub external_package_initializer: Arc<dyn ExternalPackageInitializer>,
    pub file_path_configuration: Arc<FilePathConfiguration>,
    pub file_path_displayer: Arc<dyn FilePathDisplayer>,
    pub file_system: Arc<dyn FileSystem>,
    pub build_script_compiler: Arc<dyn BuildScriptCompiler>,
    pub module_builder: Arc<dyn ModuleBuilder>,
    pub package_configuration_reader: Arc<dyn PackageConfigurationReader>,
}
