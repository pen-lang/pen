use crate::debug::Position;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Variable {
    id: usize,
    position: Position,
}

impl Variable {
    pub fn new(id: usize, position: Position) -> Self {
        Self { id, position }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
