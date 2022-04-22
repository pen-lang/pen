use crate::function::Function;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Module {
    path: String,
    functions: Vec<Function>,
}

impl Module {
    pub fn new(path: impl Into<String>, functions: Vec<Function>) -> Self {
        Self {
            path: path.into(),
            functions,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn functions(&self) -> &[Function] {
        &self.functions
    }
}
