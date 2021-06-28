mod error;
mod import_renamer;
mod module_compiler;
mod utilities;

use crate::{
    ast,
    hir::{
        self,
        analysis::{definition_qualifier, type_qualifier},
    },
    interface,
};
use error::CompileError;
use std::collections::HashMap;

pub fn compile(
    module: &ast::Module,
    prefix: &str,
    module_interfaces: &HashMap<ast::ModulePath, interface::Module>,
) -> Result<hir::Module, CompileError> {
    // TODO Do not pass module interfaces to a module compiler.
    let module = module_compiler::compile(module, module_interfaces)?;
    let module = import_renamer::rename(&module, module_interfaces);

    let module = definition_qualifier::qualify(&module, prefix);
    let module = type_qualifier::qualify(&module, prefix);

    Ok(module)
}
