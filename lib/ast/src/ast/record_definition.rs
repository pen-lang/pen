use crate::types;
use position::Position;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordDefinition {
    name: String,
    fields: Vec<types::RecordField>,
    position: Position,
}

impl RecordDefinition {
    pub fn new(
        name: impl Into<String>,
        fields: Vec<types::RecordField>,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            fields,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fields(&self) -> &[types::RecordField] {
        &self.fields
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
