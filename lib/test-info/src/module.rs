use crate::function::Function;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Module {
    path: String,
    functions: BTreeMap<String, Function>,
}

impl Module {
    pub fn new(path: impl Into<String>, functions: BTreeMap<String, Function>) -> Self {
        Self {
            path: path.into(),
            functions,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn functions(&self) -> &BTreeMap<String, Function> {
        &self.functions
    }
}
