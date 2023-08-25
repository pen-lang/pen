use super::{
    BuildScriptCompiler, BuildScriptDependencyCompiler, BuildScriptRunner, CommandRunner,
    ExternalPackageInitializer, FilePathConfiguration, FilePathDisplayer, FileSystem,
    PackageConfigurationReader, PackageConfigurationWriter, TestLinker,
};
use std::rc::Rc;

// Infrastructure is agnostic about the following concepts and their information
// should be passed through function arguments instead.
//
// - Output directory
// - Main package directory
pub struct Infrastructure {
    pub build_script_dependency_compiler: Rc<dyn BuildScriptDependencyCompiler>,
    pub external_package_initializer: Rc<dyn ExternalPackageInitializer>,
    pub file_path_configuration: Rc<FilePathConfiguration>,
    pub file_path_displayer: Rc<dyn FilePathDisplayer>,
    pub file_system: Rc<dyn FileSystem>,
    pub build_script_compiler: Rc<dyn BuildScriptCompiler>,
    pub build_script_runner: Rc<dyn BuildScriptRunner>,
    pub package_configuration_reader: Rc<dyn PackageConfigurationReader>,
    pub package_configuration_writer: Rc<dyn PackageConfigurationWriter>,
    pub command_runner: Rc<dyn CommandRunner>,
    pub test_linker: Rc<dyn TestLinker>,
}
