use crate::{
    hir::Module,
    types::{self, Type},
};
use std::collections::HashMap;

pub struct TypeContext {
    records: HashMap<String, Vec<types::RecordElement>>,
    types: HashMap<String, Type>,
}

impl TypeContext {
    pub fn new(module: &Module) -> Self {
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
        }
    }

    pub fn records(&self) -> &HashMap<String, Vec<types::RecordElement>> {
        &self.records
    }

    pub fn types(&self) -> &HashMap<String, Type> {
        &self.types
    }
}
