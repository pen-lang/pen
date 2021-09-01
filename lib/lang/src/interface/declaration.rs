
use crate::types;
use position::Position;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct Declaration {
    name: String,
    original_name: String,
    type_: types::Function,
    position: Position,
}

impl Declaration {
    pub fn new(
        name: impl Into<String>,
        original_name: impl Into<String>,
        type_: types::Function,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            original_name: original_name.into(),
            type_,
            position,
        }
    }

    #[cfg(test)]
    pub fn without_source(
        name: impl Into<String>,
        type_: types::Function,
        position: Position,
    ) -> Self {
        Self::new(name, "", type_, position)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn original_name(&self) -> &str {
        &self.original_name
    }

    pub fn type_(&self) -> &types::Function {
        &self.type_
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
