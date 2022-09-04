use super::expression::Expression;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct StringConcatenation {
    operands: Arc<Vec<Expression>>,
}

impl StringConcatenation {
    pub fn new(operands: Vec<Expression>) -> Self {
        Self {
            operands: operands.into(),
        }
    }

    pub fn operands(&self) -> &[Expression] {
        &self.operands
    }
}
