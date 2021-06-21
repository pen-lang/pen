use crate::package_builder::ModuleTarget;

pub trait ModuleBuildScriptCompiler {
    fn build(&self, module_targets: &[ModuleTarget]) -> String;
}
