use super::Type;
use crate::debug::Position;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct List {
    element: Arc<Type>,
    position: Position,
}

impl List {
    pub fn new(element: impl Into<Type>, position: Position) -> Self {
        Self {
            element: Arc::new(element.into()),
            position,
        }
    }

    pub fn element(&self) -> &Type {
        &self.element
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
