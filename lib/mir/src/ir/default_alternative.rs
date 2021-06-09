use super::expression::Expression;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct DefaultAlternative {
    name: String,
    expression: Arc<Expression>,
}

impl DefaultAlternative {
    pub fn new(name: impl Into<String>, expression: impl Into<Expression>) -> Self {
        Self {
            name: name.into(),
            expression: expression.into().into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
