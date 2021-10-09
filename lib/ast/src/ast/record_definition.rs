use crate::types;
use position::Position;

#[derive(Clone, Debug, Hash, PartialEq)]
pub struct RecordDefinition {
    name: String,
    elements: Vec<types::RecordField>,
    position: Position,
}

impl RecordDefinition {
    pub fn new(
        name: impl Into<String>,
        elements: Vec<types::RecordField>,
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

    pub fn elements(&self) -> &[types::RecordField] {
        &self.elements
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
