use super::AnalysisError;
use crate::types::{self, Type};
use fnv::FnvHashMap;

#[derive(Debug)]
pub struct AnalysisContext {
    types: FnvHashMap<String, Type>,
    records: FnvHashMap<String, Vec<types::RecordField>>,
    error_type: Option<Type>,
}

impl AnalysisContext {
    pub fn new(
        types: FnvHashMap<String, Type>,
        records: FnvHashMap<String, Vec<types::RecordField>>,
        error_type: Option<Type>,
    ) -> Self {
        Self {
            types,
            records,
            error_type,
        }
    }

    pub fn types(&self) -> &FnvHashMap<String, Type> {
        &self.types
    }

    pub fn records(&self) -> &FnvHashMap<String, Vec<types::RecordField>> {
        &self.records
    }

    pub fn error_type(&self) -> Result<&Type, AnalysisError> {
        self.error_type
            .as_ref()
            .ok_or_else(|| AnalysisError::ErrorTypeUndefined)
    }
}
