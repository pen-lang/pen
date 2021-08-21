use super::{
    ApplicationLinker, BuildScriptCompiler, BuildScriptDependencyCompiler,
    ExternalPackageInitializer, FilePathConfiguration, FilePathDisplayer, FileSystem,
    PackageBuilder, PackageConfigurationReader, PackageConfigurationWriter,
};
use std::sync::Arc;

// Infrastructure is agnostic about the following concepts and their information
// should be passed through function arguments instead.
//
// - Output directory
// - Main package directory
pub struct Infrastructure {
    pub application_linker: Arc<dyn ApplicationLinker>,
    pub build_script_dependency_compiler: Arc<dyn BuildScriptDependencyCompiler>,
    pub external_package_initializer: Arc<dyn ExternalPackageInitializer>,
    pub file_path_configuration: Arc<FilePathConfiguration>,
    pub file_path_displayer: Arc<dyn FilePathDisplayer>,
    pub file_system: Arc<dyn FileSystem>,
    pub build_script_compiler: Arc<dyn BuildScriptCompiler>,
    pub package_builder: Arc<dyn PackageBuilder>,
    pub package_configuration_reader: Arc<dyn PackageConfigurationReader>,
    pub package_configuration_writer: Arc<dyn PackageConfigurationWriter>,
}
