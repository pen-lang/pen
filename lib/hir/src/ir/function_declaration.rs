use crate::types;
use position::Position;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FunctionDeclaration {
    name: String,
    type_: types::Function,
    position: Position,
}

impl FunctionDeclaration {
    pub fn new(name: impl Into<String>, type_: types::Function, position: Position) -> Self {
        Self {
            name: name.into(),
            type_,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> &types::Function {
        &self.type_
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
