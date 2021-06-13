use super::expression::Expression;
use crate::position::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    name: String,
    expression: Expression,
    position: Position,
}

impl Assignment {
    pub fn new(
        name: impl Into<String>,
        expression: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            expression: expression.into(),
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
