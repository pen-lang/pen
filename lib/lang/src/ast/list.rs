
use super::list_element::ListElement;
use crate::types::Type;
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct List {
    type_: Type,
    elements: Vec<ListElement>,
    position: Position,
}

impl List {
    pub fn new(type_: impl Into<Type>, elements: Vec<ListElement>, position: Position) -> Self {
        Self {
            type_: type_.into(),
            elements,
            position,
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn elements(&self) -> &[ListElement] {
        &self.elements
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
