use crate::types::{self, Type};
use fnv::FnvHashMap;

#[derive(Debug)]
pub struct AnalysisContext {
    types: FnvHashMap<String, Type>,
    records: FnvHashMap<String, Vec<types::RecordField>>,
}

impl AnalysisContext {
    pub fn new(
        types: FnvHashMap<String, Type>,
        records: FnvHashMap<String, Vec<types::RecordField>>,
    ) -> Self {
        Self { types, records }
    }

    pub fn types(&self) -> &FnvHashMap<String, Type> {
        &self.types
    }

    pub fn records(&self) -> &FnvHashMap<String, Vec<types::RecordField>> {
        &self.records
    }
}
