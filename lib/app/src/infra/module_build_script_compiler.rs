use crate::package_build_script_compiler::ModuleTarget;

pub trait ModuleBuildScriptCompiler {
    fn compile(&self, module_targets: &[ModuleTarget]) -> String;
}
