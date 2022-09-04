use super::expression::Expression;
use position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct StringConcatenation {
    lhs: Arc<Expression>,
    rhs: Arc<Expression>,
    position: Position,
}

impl StringConcatenation {
    pub fn new(lhs: impl Into<Expression>, rhs: impl Into<Expression>, position: Position) -> Self {
        Self {
            lhs: lhs.into().into(),
            rhs: rhs.into().into(),
            position,
        }
    }

    pub fn lhs(&self) -> &Expression {
        &self.lhs
    }

    pub fn rhs(&self) -> &Expression {
        &self.rhs
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
