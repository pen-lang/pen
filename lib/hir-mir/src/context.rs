use crate::{CompileConfiguration, CompileError};
use hir::{
    analysis::types::type_collector,
    ir::*,
    types::{self, Type},
};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct CompileContext {
    types: BTreeMap<String, Type>,
    records: BTreeMap<String, Vec<types::RecordField>>,
    configuration: Option<CompileConfiguration>,
}

impl CompileContext {
    pub fn new(module: &Module, configuration: Option<CompileConfiguration>) -> Self {
        Self {
            types: type_collector::collect(module),
            records: module
                .type_definitions()
                .iter()
                .map(|definition| (definition.name().into(), definition.fields().to_vec()))
                .collect(),
            configuration,
        }
    }

    #[cfg(test)]
    pub fn dummy(
        types: BTreeMap<String, Type>,
        records: BTreeMap<String, Vec<types::RecordField>>,
    ) -> Self {
        use super::compile_configuration::COMPILE_CONFIGURATION;

        Self {
            types,
            records,
            configuration: COMPILE_CONFIGURATION.clone().into(),
        }
    }

    pub fn types(&self) -> &BTreeMap<String, Type> {
        &self.types
    }

    pub fn records(&self) -> &BTreeMap<String, Vec<types::RecordField>> {
        &self.records
    }

    pub fn configuration(&self) -> Result<&CompileConfiguration, CompileError> {
        self.configuration
            .as_ref()
            .ok_or(CompileError::CompileConfigurationNotProvided)
    }
}
