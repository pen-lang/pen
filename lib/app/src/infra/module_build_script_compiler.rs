use super::FilePath;
use super::ModuleTarget;

pub trait ModuleBuildScriptCompiler {
    fn compile(
        &self,
        module_targets: &[ModuleTarget],
        child_build_script_files: &[FilePath],
    ) -> String;
}
