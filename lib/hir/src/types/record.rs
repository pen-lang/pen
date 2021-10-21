use position::Position;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
pub struct Record {
    name: String,
    position: Position,
}

impl Record {
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
}
