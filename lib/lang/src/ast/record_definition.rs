use crate::types;
use position::Position;

#[derive(Clone, Debug, Hash, PartialEq)]
pub struct RecordDefinition {
    name: String,
    elements: Vec<types::RecordElement>,
    position: Position,
}

impl RecordDefinition {
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
