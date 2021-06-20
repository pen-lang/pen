use super::expression::Expression;
use crate::position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct NotOperation {
    expression: Arc<Expression>,
    position: Position,
}

impl NotOperation {
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
