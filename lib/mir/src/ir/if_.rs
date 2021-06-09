use super::expression::Expression;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct If {
    condition: Arc<Expression>,
    then: Arc<Expression>,
    else_: Arc<Expression>,
}

impl If {
    pub fn new(
        condition: impl Into<Expression>,
        then: impl Into<Expression>,
        else_: impl Into<Expression>,
    ) -> Self {
        Self {
            condition: Arc::new(condition.into()),
            then: Arc::new(then.into()),
            else_: Arc::new(else_.into()),
        }
    }

    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    pub fn then(&self) -> &Expression {
        &self.then
    }

    pub fn else_(&self) -> &Expression {
        &self.else_
    }
}
