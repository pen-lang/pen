use position::Position;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Function {
    name: String,
    foreign_name: String,
    position: Position,
}

impl Function {
    pub fn new(
        name: impl Into<String>,
        foreign_name: impl Into<String>,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            foreign_name: foreign_name.into(),
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn foreign_name(&self) -> &str {
        &self.foreign_name
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
