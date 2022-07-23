use crate::NumberRepresentation;
use position::Position;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Number {
    value: NumberRepresentation,
    position: Position,
}

impl Number {
    pub fn new(value: NumberRepresentation, position: Position) -> Self {
        Self { value, position }
    }

    pub fn value(&self) -> &NumberRepresentation {
        &self.value
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
