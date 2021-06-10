use crate::debug::Position;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Unknown {
    position: Position,
}

impl Unknown {
    pub fn new(position: Position) -> Self {
        Self { position }
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
