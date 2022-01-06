use crate::CompileConfiguration;
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
    compile_configuration: CompileConfiguration,
}

impl CompileContext {
    pub fn new(module: &Module, compile_configuration: CompileConfiguration) -> Self {
        Self {
            types: type_collector::collect(module),
            records: module
                .type_definitions()
                .iter()
                .map(|definition| (definition.name().into(), definition.fields().to_vec()))
                .collect(),
            compile_configuration,
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
            compile_configuration: COMPILE_CONFIGURATION.clone(),
        }
    }

    pub fn types(&self) -> &BTreeMap<String, Type> {
        &self.types
    }

    pub fn records(&self) -> &BTreeMap<String, Vec<types::RecordField>> {
        &self.records
    }

    pub fn compile_configuration(&self) -> &CompileConfiguration {
        &self.compile_configuration
    }
}
