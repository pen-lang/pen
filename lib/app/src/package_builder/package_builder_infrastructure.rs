use crate::infra::{FilePathConfiguration, FileSystem, ModuleBuildScriptCompiler, ModuleBuilder};
use std::sync::Arc;

pub struct PackageBuilderInfrastructure {
    pub module_builder: Arc<dyn ModuleBuilder>,
    pub module_build_script_compiler: Arc<dyn ModuleBuildScriptCompiler>,
    pub file_system: Arc<dyn FileSystem>,
    pub file_path_configuration: Arc<FilePathConfiguration>,
}
