use super::expression::Expression;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct MarkSync {
    expression: Arc<Expression>,
}

impl MarkSync {
    pub fn new(expression: impl Into<Expression>) -> Self {
        Self {
            expression: expression.into().into(),
        }
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
