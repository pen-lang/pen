use crate::{ir::*, types};
use fnv::FnvHashMap;
use std::cell::Cell;

#[derive(Debug)]
pub struct Context<'a> {
    name_index: Cell<usize>,
    record_fields: FnvHashMap<&'a str, &'a types::RecordBody>,
}

impl<'a> Context<'a> {
    pub fn new(module: &'a Module) -> Self {
        Self {
            name_index: Cell::new(0),
            record_fields: module
                .type_definitions()
                .iter()
                .map(|definition| (definition.name(), definition.type_()))
                .collect(),
        }
    }

    pub fn generate_name(&self) -> String {
        let index = self.name_index.get();

        self.name_index.set(index + 1);

        format!("anf:v:{index}")
    }

    pub fn record_fields(&self) -> &FnvHashMap<&'a str, &'a types::RecordBody> {
        &self.record_fields
    }
}
