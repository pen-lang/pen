use crate::{CompileConfiguration, CompileError};
use fnv::FnvHashMap;
use hir::{
    analysis::types::type_collector,
    ir::*,
    types::{self, Type},
};

#[derive(Debug)]
pub struct CompileContext {
    types: FnvHashMap<String, Type>,
    records: FnvHashMap<String, Vec<types::RecordField>>,
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
        types: FnvHashMap<String, Type>,
        records: FnvHashMap<String, Vec<types::RecordField>>,
    ) -> Self {
        use super::compile_configuration::COMPILE_CONFIGURATION;

        Self {
            types,
            records,
            configuration: COMPILE_CONFIGURATION.clone().into(),
        }
    }

    pub fn types(&self) -> &FnvHashMap<String, Type> {
        &self.types
    }

    pub fn records(&self) -> &FnvHashMap<String, Vec<types::RecordField>> {
        &self.records
    }

    pub fn configuration(&self) -> Result<&CompileConfiguration, CompileError> {
        self.configuration
            .as_ref()
            .ok_or(CompileError::CompileConfigurationNotProvided)
    }
}
