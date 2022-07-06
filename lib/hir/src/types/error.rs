use position::Position;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Error {
    position: Position,
}

impl Error {
    pub fn new(position: Position) -> Self {
        Self { position }
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn set_position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }
}
