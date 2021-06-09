use super::{definition::Definition, expression::Expression};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct LetRecursive {
    definition: Arc<Definition>,
    expression: Arc<Expression>,
}

impl LetRecursive {
    pub fn new(definition: Definition, expression: impl Into<Expression>) -> Self {
        Self {
            definition: definition.into(),
            expression: Arc::new(expression.into()),
        }
    }

    pub fn definition(&self) -> &Definition {
        &self.definition
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
