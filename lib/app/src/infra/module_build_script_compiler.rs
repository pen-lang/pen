use crate::package_builder::ModuleTarget;

pub trait ModuleBuildScriptCompiler {
    fn compile(&self, module_targets: &[ModuleTarget]) -> String;
}
