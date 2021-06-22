use crate::infra::{
    ExternalPackageInitializer, FilePathConfiguration, FileSystem, ModuleBuildScriptCompiler,
    PackageConfigurationReader,
};
use std::sync::Arc;

pub struct PackageInitializerInfrastructure {
    pub external_package_initializer: Arc<dyn ExternalPackageInitializer>,
    pub package_configuration_reader: Arc<dyn PackageConfigurationReader>,
    pub module_build_script_compiler: Arc<dyn ModuleBuildScriptCompiler>,
    pub file_system: Arc<dyn FileSystem>,
    pub file_path_configuration: Arc<FilePathConfiguration>,
}
