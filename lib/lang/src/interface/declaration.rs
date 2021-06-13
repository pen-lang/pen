use crate::types;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct Declaration {
    name: String,
    type_: types::Function,
}

impl Declaration {
    pub fn new(name: impl Into<String>, type_: types::Function) -> Self {
        Self {
            name: name.into(),
            type_,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> &types::Function {
        &self.type_
    }
}
