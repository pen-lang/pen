use super::expression::Expression;
use position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct If {
    condition: Arc<Expression>,
    then: Arc<Expression>,
    else_: Arc<Expression>,
    position: Position,
}

impl If {
    pub fn new(
        condition: impl Into<Expression>,
        then: impl Into<Expression>,
        else_: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            condition: Arc::new(condition.into()),
            then: then.into().into(),
            else_: else_.into().into(),
            position,
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

    pub fn position(&self) -> &Position {
        &self.position
    }
}
