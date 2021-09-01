
use super::expression::Expression;
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct Statement {
    name: Option<String>,
    expression: Expression,
    position: Position,
}

impl Statement {
    pub fn new(
        name: Option<String>,
        expression: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            name,
            expression: expression.into(),
            position,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
