use super::map_element::MapElement;
use crate::types::Type;
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct Map {
    key_type: Type,
    value_type: Type,
    elements: Vec<MapElement>,
    position: Position,
}

impl Map {
    pub fn new(
        key_type: impl Into<Type>,
        value_type: impl Into<Type>,
        elements: Vec<MapElement>,
        position: Position,
    ) -> Self {
        Self {
            key_type: key_type.into(),
            value_type: value_type.into(),
            elements,
            position,
        }
    }

    pub fn key_type(&self) -> &Type {
        &self.key_type
    }

    pub fn value_type(&self) -> &Type {
        &self.value_type
    }

    pub fn elements(&self) -> &[MapElement] {
        &self.elements
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
