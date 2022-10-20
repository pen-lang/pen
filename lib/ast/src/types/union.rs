use super::Type;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Union {
    lhs: Rc<Type>,
    rhs: Rc<Type>,
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
}
