use super::Type;
use position::Position;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Union(Arc<UnionInner>);

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
struct UnionInner {
    lhs: Type,
    rhs: Type,
    position: Position,
}

impl Union {
    pub fn new(lhs: impl Into<Type>, rhs: impl Into<Type>, position: Position) -> Self {
        Self(
            UnionInner {
                lhs: lhs.into().into(),
                rhs: rhs.into().into(),
                position,
            }
            .into(),
        )
    }

    pub fn lhs(&self) -> &Type {
        &self.0.lhs
    }

    pub fn rhs(&self) -> &Type {
        &self.0.rhs
    }

    pub fn position(&self) -> &Position {
        &self.0.position
    }

    pub fn set_position(&self, position: Position) -> Self {
        Self(
            UnionInner {
                lhs: self.0.lhs.clone(),
                rhs: self.0.rhs.clone(),
                position,
            }
            .into(),
        )
    }
}
