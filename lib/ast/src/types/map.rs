use super::Type;
use position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Map {
    key: Arc<Type>,
    value: Arc<Type>,
    position: Position,
}

impl Map {
    pub fn new(key: impl Into<Type>, value: impl Into<Type>, position: Position) -> Self {
        Self {
            key: Arc::new(key.into()),
            value: Arc::new(value.into()),
            position,
        }
    }

    pub fn key(&self) -> &Type {
        &self.key
    }

    pub fn value(&self) -> &Type {
        &self.value
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
