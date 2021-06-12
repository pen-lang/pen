use super::error::CompileError;
use crate::hir::{Module, ModuleInterface};

pub fn compile(_module: &Module) -> Result<ModuleInterface, CompileError> {
    Ok(ModuleInterface::new(vec![], vec![], vec![]))
}
