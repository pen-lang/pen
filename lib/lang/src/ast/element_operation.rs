use super::{expression::Expression, unary_operator::UnaryOperator};
use crate::position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct ElementOperation {
    expression: Arc<Expression>,
    name: String,
    position: Position,
}

impl ElementOperation {
    pub fn new(
        expression: impl Into<Expression>,
        name: impl Into<String>,
        position: Position,
    ) -> Self {
        Self {
            expression: expression.into().into(),
            name: name.into(),
            position,
        }
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
