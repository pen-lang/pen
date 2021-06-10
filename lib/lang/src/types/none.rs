use crate::debug::Position;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct None {
    position: Position,
}

impl None {
    pub fn new(position: Position) -> Self {
        Self { position }
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
