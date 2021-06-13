use super::{expression::Expression, Block};
use crate::position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct If {
    condition: Arc<Expression>,
    then: Arc<Block>,
    else_: Arc<Block>,
    position: Position,
}

impl If {
    pub fn new(
        condition: impl Into<Expression>,
        then: Block,
        else_: Block,
        position: Position,
    ) -> Self {
        Self {
            condition: Arc::new(condition.into()),
            then: then.into(),
            else_: else_.into(),
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

    pub fn position(&self) -> &Position {
        &self.position
    }
}
