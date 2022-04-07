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
                    definition.fields().to_vec(),
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
        module.function_declarations().to_vec(),
        module
            .function_definitions()
            .iter()
            .map(|definition| {
                FunctionDefinition::new(
                    definition.name(),
                    definition.original_name(),
                    definition.lambda().clone(),
                    definition.foreign_definition_configuration().cloned(),
                    true,
                    definition.position().clone(),
                )
            })
            .collect(),
        module.position().clone(),
    )
}
