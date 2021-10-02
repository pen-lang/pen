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
        ffi_archive_file: &FilePath,
        package_directory: &FilePath,
    ) -> Result<String, Box<dyn Error>>;

    fn compile_test_modules(
        &self,
        module_targets: &[TestModuleTarget],
        archive_file: &FilePath,
        package_directory: &FilePath,
    ) -> Result<String, Box<dyn Error>>;

    fn compile_application(
        &self,
        system_package_directory: &FilePath,
        archive_files: &[FilePath],
        application_file: &FilePath,
    ) -> Result<String, Box<dyn Error>>;

    fn compile_test(
        &self,
        test_package_directory: &FilePath,
        test_interface_file: &FilePath,
        archive_files: &[FilePath],
        test_file: &FilePath,
    ) -> Result<String, Box<dyn Error>>;

    fn compile_external(
        &self,
        module_targets: &[ModuleTarget],
        archive_file: &FilePath,
        ffi_archive_file: &FilePath,
        package_directory: &FilePath,
    ) -> Result<String, Box<dyn Error>>;

    fn compile_prelude(
        &self,
        module_targets: &[ModuleTarget],
        archive_file: &FilePath,
        ffi_archive_file: &FilePath,
        package_directory: &FilePath,
    ) -> Result<String, Box<dyn Error>>;
}
