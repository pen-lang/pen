use position::Position;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Reference {
    name: String,
    position: Position,
}

impl Reference {
    pub fn new(name: impl Into<String>, position: Position) -> Self {
        Self {
            name: name.into(),
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn set_position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }
}
