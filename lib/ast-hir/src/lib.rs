mod error;
mod import_compiler;
mod imported_module;
mod module_compiler;
mod module_prefix_collector;
mod name_qualifier;
mod number_compiler;
mod prelude_module_modifier;
mod string_compiler;

use error::CompileError;
use fnv::FnvHashMap;
use hir::{
    analysis::{definition_qualifier, type_qualifier},
    ir,
};
use imported_module::ImportedModule;

pub fn compile(
    module: &ast::Module,
    prefix: &str,
    module_interfaces: &FnvHashMap<ast::ModulePath, interface::Module>,
    prelude_module_interfaces: &[interface::Module],
) -> Result<ir::Module, CompileError> {
    let imported_modules = module
        .imports()
        .iter()
        .map(|import| {
            Ok(ImportedModule::new(
                module_interfaces
                    .get(import.module_path())
                    .ok_or_else(|| CompileError::ModuleNotFound(import.module_path().clone()))?
                    .clone(),
                module_prefix_collector::calculate(import),
                import.unqualified_names().iter().cloned().collect(),
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let module = module_compiler::compile(module)?;
    let module = import_compiler::compile(&module, &imported_modules, prelude_module_interfaces);

    let module = definition_qualifier::qualify(&module, prefix);
    let module = type_qualifier::qualify(&module, prefix);

    Ok(module)
}

pub fn compile_prelude(module: &ast::Module, prefix: &str) -> Result<ir::Module, CompileError> {
    let module = module_compiler::compile(module)?;
    let module = definition_qualifier::qualify(&module, prefix);
    let module = type_qualifier::qualify(&module, prefix);
    let module = prelude_module_modifier::modify(&module);

    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hir::test::ModuleFake;
    use position::{test::PositionFake, Position};

    #[test]
    fn compile_empty_module() {
        assert_eq!(
            compile(
                &ast::Module::new(vec![], vec![], vec![], vec![], Position::fake()),
                "",
                &Default::default(),
                &[],
            ),
            Ok(ir::Module::empty()),
        );
    }

    #[test]
    fn compile_module_with_() {
        let path = ast::InternalModulePath::new(vec!["Foo".into()]);

        assert_eq!(
            compile(
                &ast::Module::new(
                    vec![ast::Import::new(path.clone(), None, vec![])],
                    vec![],
                    vec![],
                    vec![],
                    Position::fake()
                ),
                "",
                &Default::default(),
                &[],
            ),
            Err(CompileError::ModuleNotFound(path.into())),
        );
    }
}
