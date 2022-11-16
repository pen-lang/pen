use super::{FilePath, MainModuleTarget, ModuleTarget, TestModuleTarget};
use std::error::Error;

pub trait BuildScriptCompiler {
    fn compile_main(
        &self,
        prelude_interface_files: &[FilePath],
        output_directory: &FilePath,
        target_triple: Option<&str>,
        child_build_script_files: &[FilePath],
    ) -> Result<String, Box<dyn Error>>;

    fn compile_modules(
        &self,
        module_targets: &[ModuleTarget],
        main_module_target: Option<&MainModuleTarget>,
        archive_file: &FilePath,
        package_directory: &FilePath,
    ) -> Result<String, Box<dyn Error>>;

    fn compile_test_modules(
        &self,
        module_targets: &[TestModuleTarget],
        archive_file: &FilePath,
        test_information_file: &FilePath,
    ) -> Result<String, Box<dyn Error>>;

    fn compile_application(
        &self,
        system_package_directories: &[FilePath],
        archive_files: &[FilePath],
        application_file: &FilePath,
    ) -> Result<String, Box<dyn Error>>;

    fn compile_test(
        &self,
        archive_files: &[FilePath],
        test_information_file: &FilePath,
        test_file: &FilePath,
    ) -> Result<String, Box<dyn Error>>;

    fn compile_external_package(
        &self,
        module_targets: &[ModuleTarget],
        archive_file: &FilePath,
        package_directory: &FilePath,
        package_name: &str,
    ) -> Result<String, Box<dyn Error>>;

    fn compile_prelude_package(
        &self,
        module_targets: &[ModuleTarget],
        archive_file: &FilePath,
        package_directory: &FilePath,
        package_name: &str,
    ) -> Result<String, Box<dyn Error>>;
}
