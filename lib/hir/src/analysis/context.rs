use crate::{
    analysis::type_collector,
    ir::*,
    types::{self, Type},
};
use fnv::FnvHashMap;

#[derive(Debug)]
pub struct AnalysisContext {
    types: FnvHashMap<String, Type>,
    records: FnvHashMap<String, Vec<types::RecordField>>,
    error_type: Type,
}

impl AnalysisContext {
    pub fn new(module: &Module, error_type: impl Into<Type>) -> Self {
        Self {
            types: type_collector::collect(module),
            records: module
                .type_definitions()
                .iter()
                .map(|definition| (definition.name().into(), definition.fields().to_vec()))
                .collect(),
            error_type: error_type.into(),
        }
    }

    pub fn types(&self) -> &FnvHashMap<String, Type> {
        &self.types
    }

    pub fn records(&self) -> &FnvHashMap<String, Vec<types::RecordField>> {
        &self.records
    }

    pub fn error_type(&self) -> &Type {
        &self.error_type
    }
}
