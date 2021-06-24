use super::{FilePath, ModuleTarget};
use std::error::Error;

pub trait ModuleBuildScriptCompiler {
    fn compile(
        &self,
        module_targets: &[ModuleTarget],
        child_build_script_files: &[FilePath],
    ) -> Result<String, Box<dyn Error>>;
}
