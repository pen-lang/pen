use super::Type;
use position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Union {
    lhs: Arc<Type>,
    rhs: Arc<Type>,
    position: Position,
}

impl Union {
    pub fn new(lhs: impl Into<Type>, rhs: impl Into<Type>, position: Position) -> Self {
        Self {
            lhs: lhs.into().into(),
            rhs: rhs.into().into(),
            position,
        }
    }

    pub fn lhs(&self) -> &Type {
        &self.lhs
    }

    pub fn rhs(&self) -> &Type {
        &self.rhs
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn set_position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }
}
