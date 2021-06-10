use crate::debug::Position;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Number {
    position: Position,
}

impl Number {
    pub fn new(position: Position) -> Self {
        Self { position }
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
