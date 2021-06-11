use super::{Expression, Statement};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    statements: Vec<Statement>,
    result: Arc<Expression>,
}

impl Block {
    pub fn new(statements: Vec<Statement>, result: impl Into<Expression>) -> Self {
        Self {
            statements,
            result: result.into().into(),
        }
    }

    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }

    pub fn result(&self) -> &Expression {
        &self.result
    }
}
