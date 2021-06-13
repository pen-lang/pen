use super::lambda::Lambda;
use crate::position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct Definition {
    name: String,
    lambda: Lambda,
    public: bool,
    position: Position,
}

impl Definition {
    pub fn new(name: impl Into<String>, lambda: Lambda, public: bool, position: Position) -> Self {
        Self {
            name: name.into(),
            lambda,
            public,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn lambda(&self) -> &Lambda {
        &self.lambda
    }

    pub fn is_public(&self) -> bool {
        self.public
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
