use super::expression::Expression;
use crate::position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct TryOperation {
    expression: Arc<Expression>,
    position: Position,
}

impl TryOperation {
    pub fn new(expression: impl Into<Expression>, position: Position) -> Self {
        Self {
            expression: expression.into().into(),
            position,
        }
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
