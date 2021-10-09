use super::RecordField;
use crate::types::Type;
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordConstruction {
    type_: Type,
    fields: Vec<RecordField>,
    position: Position,
}

impl RecordConstruction {
    pub fn new(type_: impl Into<Type>, fields: Vec<RecordField>, position: Position) -> Self {
        Self {
            type_: type_.into(),
            fields,
            position,
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn fields(&self) -> &[RecordField] {
        &self.fields
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
