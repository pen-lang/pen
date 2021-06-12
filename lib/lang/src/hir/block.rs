use super::{Assignment, Expression};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    assignments: Vec<Assignment>,
    expression: Arc<Expression>,
}

impl Block {
    pub fn new(assignments: Vec<Assignment>, expression: impl Into<Expression>) -> Self {
        Self {
            assignments,
            expression: expression.into().into(),
        }
    }

    pub fn assignments(&self) -> &[Assignment] {
        &self.assignments
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
