mod error;
mod import_renamer;
mod module_compiler;

use crate::{ast, hir, interface};
use error::CompileError;
use std::collections::HashMap;

pub fn compile(
    module: &ast::Module,
    prefix: &str,
    module_interfaces: &HashMap<ast::ModulePath, interface::Module>,
) -> Result<hir::Module, CompileError> {
    let module = module_compiler::compile(module, module_interfaces)?;
    let module = hir::analysis::definition_qualifier::qualify(&module, prefix);
    let module = hir::analysis::type_qualifier::qualify(&module, prefix);
    let module = import_renamer::rename(&module, module_interfaces);

    Ok(module)
}
