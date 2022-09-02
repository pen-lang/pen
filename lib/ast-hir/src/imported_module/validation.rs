use crate::{error::CompileError, imported_module::ImportedModule};
use fnv::FnvHashSet;

pub fn validate(module: &ImportedModule) -> Result<(), CompileError> {
    let names = module
        .interface()
        .type_definitions()
        .iter()
        .map(|definition| definition.original_name())
        .chain(
            module
                .interface()
                .type_aliases()
                .iter()
                .map(|alias| alias.original_name()),
        )
        .chain(
            module
                .interface()
                .function_declarations()
                .iter()
                .map(|declaration| declaration.original_name()),
        )
        .collect::<FnvHashSet<_>>();

    for name in module.unqualified_names() {
        if !names.contains(&**name) {
            return Err(CompileError::NameNotFound(name.into()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_undefined_name() {
        assert_eq!(
            validate(&ImportedModule::new(
                interface::Module::new(vec![], vec![], vec![]),
                "",
                ["foo".into()].into_iter().collect()
            )),
            Err(CompileError::NameNotFound("foo".into()))
        );
    }
}
