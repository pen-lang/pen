use super::expression::Expression;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordDeconstruction {
    expression: Rc<Expression>,
    name: String,
    position: Position,
}

impl RecordDeconstruction {
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
