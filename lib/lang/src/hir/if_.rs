use super::{expression::Expression, Block};
use crate::{position::Position, types::Type};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct If {
    condition: Arc<Expression>,
    then: Arc<Block>,
    else_: Arc<Block>,
    result_type: Option<Type>,
    position: Position,
}

impl If {
    pub fn new(
        condition: impl Into<Expression>,
        then: Block,
        else_: Block,
        result_type: Option<Type>,
        position: Position,
    ) -> Self {
        Self {
            condition: Arc::new(condition.into()),
            then: then.into(),
            else_: else_.into(),
            result_type,
            position,
        }
    }

    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    pub fn then(&self) -> &Block {
        &self.then
    }

    pub fn else_(&self) -> &Block {
        &self.else_
    }

    pub fn result_type(&self) -> Option<&Type> {
        self.result_type.as_ref()
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
