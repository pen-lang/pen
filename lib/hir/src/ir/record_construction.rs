use super::RecordField;
use crate::types::Type;
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordConstruction {
    type_: Type,
    elements: Vec<RecordField>,
    position: Position,
}

impl RecordConstruction {
    pub fn new(type_: impl Into<Type>, elements: Vec<RecordField>, position: Position) -> Self {
        Self {
            type_: type_.into(),
            elements,
            position,
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn elements(&self) -> &[RecordField] {
        &self.elements
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
