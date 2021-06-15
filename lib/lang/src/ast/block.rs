use super::{Expression, Statement};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    statements: Vec<Statement>,
    expression: Arc<Expression>,
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
