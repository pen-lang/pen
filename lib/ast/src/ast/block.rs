use super::{Expression, Statement};
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    statements: Vec<Statement>,
    expression: Rc<Expression>,
    position: Position,
}

impl Block {
    pub fn new(
        statements: Vec<Statement>,
        expression: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            statements,
            expression: expression.into().into(),
            position,
        }
    }

    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
