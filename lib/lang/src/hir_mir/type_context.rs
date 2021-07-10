use super::{
    error_type_configuration::ErrorTypeConfiguration,
    list_type_configuration::ListTypeConfiguration,
    string_type_configuration::StringTypeConfiguration,
};
use crate::{
    hir::Module,
    types::{self, Type},
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct TypeContext {
    records: HashMap<String, Vec<types::RecordElement>>,
    types: HashMap<String, Type>,
    list_type_configuration: ListTypeConfiguration,
    string_type_configuration: StringTypeConfiguration,
    error_type_configuration: ErrorTypeConfiguration,
}

impl TypeContext {
    pub fn new(
        module: &Module,
        list_type_configuration: &ListTypeConfiguration,
        string_type_configuration: &StringTypeConfiguration,
        error_type_configuration: &ErrorTypeConfiguration,
    ) -> Self {
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
            string_type_configuration: string_type_configuration.clone(),
            error_type_configuration: error_type_configuration.clone(),
        }
    }

    #[cfg(test)]
    pub fn dummy(
        records: HashMap<String, Vec<types::RecordElement>>,
        types: HashMap<String, Type>,
    ) -> Self {
        use super::{
            error_type_configuration::ERROR_TYPE_CONFIGURATION,
            list_type_configuration::LIST_TYPE_CONFIGURATION,
            string_type_configuration::STRING_TYPE_CONFIGURATION,
        };

        Self {
            records,
            types,
            list_type_configuration: LIST_TYPE_CONFIGURATION.clone(),
            string_type_configuration: STRING_TYPE_CONFIGURATION.clone(),
            error_type_configuration: ERROR_TYPE_CONFIGURATION.clone(),
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

    pub fn string_type_configuration(&self) -> &StringTypeConfiguration {
        &self.string_type_configuration
    }

    pub fn error_type_configuration(&self) -> &ErrorTypeConfiguration {
        &self.error_type_configuration
    }
}
