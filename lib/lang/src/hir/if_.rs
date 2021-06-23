use super::expression::Expression;
use crate::{position::Position, types::Type};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct If {
    condition: Arc<Expression>,
    then: Arc<Expression>,
    else_: Arc<Expression>,
    result_type: Option<Type>,
    position: Position,
}

impl If {
    pub fn new(
        condition: impl Into<Expression>,
        then: impl Into<Expression>,
        else_: impl Into<Expression>,
        result_type: Option<Type>,
        position: Position,
    ) -> Self {
        Self {
            condition: Arc::new(condition.into()),
            then: then.into().into(),
            else_: else_.into().into(),
            result_type,
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

    pub fn result_type(&self) -> Option<&Type> {
        self.result_type.as_ref()
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
