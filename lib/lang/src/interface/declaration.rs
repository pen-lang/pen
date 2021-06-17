use crate::{position::Position, types};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct Declaration {
    name: String,
    type_: types::Function,
    position: Position,
}

impl Declaration {
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
