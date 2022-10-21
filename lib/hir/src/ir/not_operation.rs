use super::expression::Expression;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct NotOperation {
    expression: Rc<Expression>,
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
