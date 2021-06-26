use super::RecordElement;
use crate::{position::Position, types::Type};

#[derive(Clone, Debug, PartialEq)]
pub struct RecordConstruction {
    type_: Type,
    elements: Vec<RecordElement>,
    position: Position,
}

impl RecordConstruction {
    pub fn new(type_: impl Into<Type>, elements: Vec<RecordElement>, position: Position) -> Self {
        Self {
            type_: type_.into(),
            elements,
            position,
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn elements(&self) -> &[RecordElement] {
        &self.elements
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
