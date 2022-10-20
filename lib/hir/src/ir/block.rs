use position::Position; use super::{Expression, Statement};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    statements: Vec<Statement>,
    expression: Rc<Expression>,
}

impl Block {
    pub fn new(statements: Vec<Statement>, expression: impl Into<Expression>) -> Self {
        Self {
            statements,
            expression: expression.into().into(),
        }
    }

    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
