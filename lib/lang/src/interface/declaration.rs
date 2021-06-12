use crate::{position::Position, types::Type};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct Declaration {
    name: String,
    type_: Type,
    position: Position,
}

impl Declaration {
    pub fn new(name: impl Into<String>, type_: impl Into<Type>, position: Position) -> Self {
        Self {
            name: name.into(),
            type_: type_.into(),
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
