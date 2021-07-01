mod error;
mod import_compiler;
mod module_canonicalizer;
mod module_compiler;
mod prelude_module_modifier;
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
    prelude_module_interfaces: &[interface::Module],
) -> Result<hir::Module, CompileError> {
    // TODO Do not pass module interfaces to a module compiler but merge them later.
    let module = module_compiler::compile(
        module,
        &module_interfaces
            .values()
            .chain(prelude_module_interfaces)
            .cloned()
            .collect::<Vec<_>>(),
    )?;
    let module = import_compiler::compile(&module, module_interfaces, prelude_module_interfaces);

    let module = definition_qualifier::qualify(&module, prefix);
    let module = type_qualifier::qualify(&module, prefix);

    Ok(module)
}

pub fn compile_prelude(module: &ast::Module, prefix: &str) -> Result<hir::Module, CompileError> {
    let module = module_compiler::compile(module, &[])?;
    let module = definition_qualifier::qualify(&module, prefix);
    let module = type_qualifier::qualify(&module, prefix);
    let module = prelude_module_modifier::modify(&module);

    Ok(module)
}
