use super::lambda::Lambda;
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct Definition {
    name: String,
    lambda: Lambda,
    foreign: bool,
    position: Position,
}

impl Definition {
    pub fn new(name: impl Into<String>, lambda: Lambda, foreign: bool, position: Position) -> Self {
        Self {
            name: name.into(),
            lambda,
            foreign,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn lambda(&self) -> &Lambda {
        &self.lambda
    }

    pub fn is_foreign(&self) -> bool {
        self.foreign
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
