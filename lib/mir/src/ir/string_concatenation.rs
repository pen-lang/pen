use super::expression::Expression;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct StringConcatenation {
    lhs: Arc<Expression>,
    rhs: Arc<Expression>,
}

impl StringConcatenation {
    pub fn new(lhs: impl Into<Expression>, rhs: impl Into<Expression>) -> Self {
        Self {
            lhs: Arc::new(lhs.into()),
            rhs: Arc::new(rhs.into()),
        }
    }

    pub fn lhs(&self) -> &Expression {
        &self.lhs
    }

    pub fn rhs(&self) -> &Expression {
        &self.rhs
    }
}
