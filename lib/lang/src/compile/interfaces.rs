use super::error::CompileError;
use crate::{hir, interface};

pub fn compile(_module: &hir::Module) -> Result<interface::Module, CompileError> {
    Ok(interface::Module::new(vec![], vec![], vec![]))
}
