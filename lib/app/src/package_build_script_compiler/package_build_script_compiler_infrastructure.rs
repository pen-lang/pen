use crate::infra::{FilePathConfiguration, FileSystem, ModuleBuildScriptCompiler};
use std::sync::Arc;

pub struct PackageBuildScriptCompilerInfrastructure {
    pub module_build_script_compiler: Arc<dyn ModuleBuildScriptCompiler>,
    pub file_system: Arc<dyn FileSystem>,
    pub file_path_configuration: Arc<FilePathConfiguration>,
}
