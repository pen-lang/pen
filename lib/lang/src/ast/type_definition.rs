use crate::{position::Position, types};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct TypeDefinition {
    name: String,
    elements: Vec<types::RecordElement>,
    position: Position,
}

impl TypeDefinition {
    pub fn new(
        name: impl Into<String>,
        elements: Vec<types::RecordElement>,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            elements,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn elements(&self) -> &[types::RecordElement] {
        &self.elements
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
