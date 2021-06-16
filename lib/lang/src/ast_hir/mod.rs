mod error;
mod module_compiler;

use crate::{ast, hir, interface};
use error::CompileError;

pub fn compile(
    module: ast::Module,
    prefix: &str,
    module_interfaces: &[interface::Module],
) -> Result<hir::Module, CompileError> {
    let module = module_compiler::compile(module, module_interfaces)?;
    let module = hir::analysis::definition_qualifier::qualify(&module, prefix);

    Ok(module)
}
