use super::Type;
use position::Position;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Function {
    arguments: Vec<Type>,
    result: Arc<Type>,
    position: Position,
}

impl Function {
    pub fn new(arguments: Vec<Type>, result: impl Into<Type>, position: Position) -> Self {
        Self {
            arguments,
            result: Arc::new(result.into()),
            position,
        }
    }

    pub fn arguments(&self) -> &[Type] {
        &self.arguments
    }

    pub fn result(&self) -> &Type {
        &self.result
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn set_position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }
}
