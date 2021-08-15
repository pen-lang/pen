use super::{FilePath, MainModuleTarget, ModuleTarget};
use std::error::Error;

pub trait BuildScriptCompiler {
    #[allow(clippy::too_many_arguments)]
    fn compile_main(
        &self,
        module_targets: &[ModuleTarget],
        main_module_target: Option<&MainModuleTarget>,
        child_build_script_files: &[FilePath],
        prelude_interface_files: &[FilePath],
        archive_file: &FilePath,
        ffi_archive_file: &FilePath,
        package_directory: &FilePath,
        output_directory: &FilePath,
        target_triple: Option<&str>,
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
