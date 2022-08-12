use super::Type;
use position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
