use super::lambda::Lambda;
use crate::position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct Definition {
    name: String,
    lambda: Lambda,
    position: Position,
}

impl Definition {
    pub fn new(name: impl Into<String>, lambda: Lambda, position: Position) -> Self {
        Self {
            name: name.into(),
            lambda,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn lambda(&self) -> &Lambda {
        &self.lambda
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
