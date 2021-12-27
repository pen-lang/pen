use super::expression::Expression;
use position::Position;
use std::sync::Arc;

// TODO Cache inferred types.
#[derive(Clone, Debug, PartialEq)]
pub struct AsyncOperation {
    expression: Arc<Expression>,
    position: Position,
}

impl AsyncOperation {
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
