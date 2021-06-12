use super::list_type_configuration::ListTypeConfiguration;
use crate::{
    hir::Module,
    types::{self, Type},
};
use std::collections::HashMap;

pub struct TypeContext {
    records: HashMap<String, Vec<types::RecordElement>>,
    types: HashMap<String, Type>,
    list_type_configuration: ListTypeConfiguration,
}

impl TypeContext {
    pub fn new(module: &Module, list_type_configuration: &ListTypeConfiguration) -> Self {
        Self {
            records: module
                .type_definitions()
                .iter()
                .map(|definition| (definition.name().into(), definition.elements().to_vec()))
                .collect(),
            types: module
                .type_definitions()
                .iter()
                .map(|definition| {
                    (
                        definition.name().into(),
                        types::Record::new(definition.name(), definition.position().clone()).into(),
                    )
                })
                .chain(
                    module
                        .type_aliases()
                        .iter()
                        .map(|alias| (alias.name().into(), alias.type_().clone())),
                )
                .collect(),
            list_type_configuration: list_type_configuration.clone(),
        }
    }

    pub fn records(&self) -> &HashMap<String, Vec<types::RecordElement>> {
        &self.records
    }

    pub fn types(&self) -> &HashMap<String, Type> {
        &self.types
    }

    pub fn list_type_configuration(&self) -> &ListTypeConfiguration {
        &self.list_type_configuration
    }
}
