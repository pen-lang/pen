use crate::hir::*;
use std::collections::HashSet;

pub fn canonicalize(module: &Module) -> Module {
    Module::new(
        module
            .type_definitions()
            .iter()
            .map(|definition| definition.name())
            .collect::<HashSet<_>>()
            .into_iter()
            .flat_map(|name| {
                module
                    .type_definitions()
                    .iter()
                    .find(|definition| definition.name() == name)
                    .cloned()
            })
            .collect(),
        module
            .type_aliases()
            .iter()
            .map(|alias| alias.name())
            .collect::<HashSet<_>>()
            .into_iter()
            .flat_map(|name| {
                module
                    .type_aliases()
                    .iter()
                    .find(|alias| alias.name() == name)
                    .cloned()
            })
            .collect(),
        module
            .declarations()
            .iter()
            .map(|declaration| declaration.name())
            .collect::<HashSet<_>>()
            .into_iter()
            .flat_map(|name| {
                module
                    .declarations()
                    .iter()
                    .find(|declaration| declaration.name() == name)
                    .cloned()
            })
            .collect(),
        module.definitions().to_vec(),
    )
}
