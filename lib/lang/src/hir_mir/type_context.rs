use super::{
    list_type_configuration::ListTypeConfiguration,
    string_type_configuration::StringTypeConfiguration,
};
use crate::{
    hir::Module,
    types::{self, Type},
};
use std::collections::HashMap;

pub struct TypeContext {
    records: HashMap<String, HashMap<String, Type>>,
    types: HashMap<String, Type>,
    list_type_configuration: ListTypeConfiguration,
    string_type_configuration: StringTypeConfiguration,
}

impl TypeContext {
    pub fn new(
        module: &Module,
        list_type_configuration: &ListTypeConfiguration,
        string_type_configuration: &StringTypeConfiguration,
    ) -> Self {
        Self {
            records: module
                .type_definitions()
                .iter()
                .map(|definition| {
                    (
                        definition.name().into(),
                        definition
                            .elements()
                            .iter()
                            .map(|element| (element.name().into(), element.type_().clone()))
                            .collect(),
                    )
                })
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
            string_type_configuration: string_type_configuration.clone(),
        }
    }

    pub fn records(&self) -> &HashMap<String, HashMap<String, Type>> {
        &self.records
    }

    pub fn types(&self) -> &HashMap<String, Type> {
        &self.types
    }

    pub fn list_type_configuration(&self) -> &ListTypeConfiguration {
        &self.list_type_configuration
    }

    pub fn string_type_configuration(&self) -> &StringTypeConfiguration {
        &self.string_type_configuration
    }
}
