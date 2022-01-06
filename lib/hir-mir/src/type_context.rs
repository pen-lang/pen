use super::{
    concurrency_configuration::ConcurrencyConfiguration,
    error_type_configuration::ErrorTypeConfiguration,
    list_type_configuration::ListTypeConfiguration,
    string_type_configuration::StringTypeConfiguration,
};
use hir::{
    analysis::types::type_collector,
    ir::*,
    types::{self, Type},
};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct TypeContext {
    types: BTreeMap<String, Type>,
    records: BTreeMap<String, Vec<types::RecordField>>,
    // TODO Consider moving those to CompileConfiguration together with ConcurrencyConfiguration.
    list_type_configuration: ListTypeConfiguration,
    string_type_configuration: StringTypeConfiguration,
    error_type_configuration: ErrorTypeConfiguration,
    concurrency_configuration: ConcurrencyConfiguration,
}

impl TypeContext {
    pub fn new(
        module: &Module,
        list_type_configuration: &ListTypeConfiguration,
        string_type_configuration: &StringTypeConfiguration,
        error_type_configuration: &ErrorTypeConfiguration,
        concurrency_configuration: &ConcurrencyConfiguration,
    ) -> Self {
        Self {
            types: type_collector::collect(module),
            records: module
                .type_definitions()
                .iter()
                .map(|definition| (definition.name().into(), definition.fields().to_vec()))
                .collect(),
            list_type_configuration: list_type_configuration.clone(),
            string_type_configuration: string_type_configuration.clone(),
            error_type_configuration: error_type_configuration.clone(),
            concurrency_configuration: concurrency_configuration.clone(),
        }
    }

    #[cfg(test)]
    pub fn dummy(
        types: BTreeMap<String, Type>,
        records: BTreeMap<String, Vec<types::RecordField>>,
    ) -> Self {
        use super::{
            concurrency_configuration::CONCURRENCY_CONFIGURATION,
            error_type_configuration::ERROR_TYPE_CONFIGURATION,
            list_type_configuration::LIST_TYPE_CONFIGURATION,
            string_type_configuration::STRING_TYPE_CONFIGURATION,
        };

        Self {
            types,
            records,
            list_type_configuration: LIST_TYPE_CONFIGURATION.clone(),
            string_type_configuration: STRING_TYPE_CONFIGURATION.clone(),
            error_type_configuration: ERROR_TYPE_CONFIGURATION.clone(),
            concurrency_configuration: CONCURRENCY_CONFIGURATION.clone(),
        }
    }

    pub fn types(&self) -> &BTreeMap<String, Type> {
        &self.types
    }

    pub fn records(&self) -> &BTreeMap<String, Vec<types::RecordField>> {
        &self.records
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

    pub fn concurrency_configuration(&self) -> &ConcurrencyConfiguration {
        &self.concurrency_configuration
    }
}
