use hir::ir::*;

pub fn modify(module: &Module) -> Module {
    Module::new(
        module
            .type_definitions()
            .iter()
            .map(|definition| {
                TypeDefinition::new(
                    definition.name(),
                    definition.original_name(),
                    definition.elements().to_vec(),
                    definition.is_open(),
                    true,
                    definition.is_external(),
                    definition.position().clone(),
                )
            })
            .collect(),
        module
            .type_aliases()
            .iter()
            .map(|alias| {
                TypeAlias::new(
                    alias.name(),
                    alias.original_name(),
                    alias.type_().clone(),
                    true,
                    alias.is_external(),
                    alias.position().clone(),
                )
            })
            .collect(),
        module.foreign_declarations().to_vec(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .map(|definition| {
                Definition::new(
                    definition.name(),
                    definition.original_name(),
                    definition.lambda().clone(),
                    definition.is_foreign(),
                    true,
                    definition.position().clone(),
                )
            })
            .collect(),
        module.position().clone(),
    )
}
