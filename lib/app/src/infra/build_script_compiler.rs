use super::{FilePath, ModuleTarget};
use std::error::Error;

pub trait BuildScriptCompiler {
    fn compile(
        &self,
        module_targets: &[ModuleTarget],
        child_build_script_files: &[FilePath],
        prelude_interface_files: &[FilePath],
        ffi_archive_file: &FilePath,
        package_directory: &FilePath,
    ) -> Result<String, Box<dyn Error>>;

    fn compile_prelude(
        &self,
        module_targets: &[ModuleTarget],
        ffi_archive_file: &FilePath,
        package_directory: &FilePath,
    ) -> Result<String, Box<dyn Error>>;
}
