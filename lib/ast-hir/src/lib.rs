mod error;
mod import_compiler;
mod module_compiler;
mod name_qualifier;
mod prelude_module_modifier;
mod singleton_record_compiler;
mod test;

use error::CompileError;
use hir::{
    analysis::ir::{definition_qualifier, type_qualifier},
    ir,
};
use lang::ast;
use std::collections::HashMap;

pub fn compile(
    module: &ast::Module,
    prefix: &str,
    module_interfaces: &HashMap<ast::ModulePath, interface::Module>,
    prelude_module_interfaces: &[interface::Module],
) -> Result<ir::Module, CompileError> {
    let module = module_compiler::compile(module)?;
    let module = import_compiler::compile(&module, module_interfaces, prelude_module_interfaces);

    let module = definition_qualifier::qualify(&module, prefix);
    let module = type_qualifier::qualify(&module, prefix);

    let module = singleton_record_compiler::compile(&module, module_interfaces);

    Ok(module)
}

pub fn compile_prelude(module: &ast::Module, prefix: &str) -> Result<ir::Module, CompileError> {
    let module = module_compiler::compile(module)?;
    let module = definition_qualifier::qualify(&module, prefix);
    let module = type_qualifier::qualify(&module, prefix);
    let module = prelude_module_modifier::modify(&module);

    Ok(module)
}
