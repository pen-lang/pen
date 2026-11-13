use super::{Block, expression::Expression};
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct IfMap {
    name: String,
    map: Rc<Expression>,
    key: Rc<Expression>,
    then: Rc<Block>,
    else_: Rc<Block>,
    position: Position,
}

impl IfMap {
    pub fn new(
        name: impl Into<String>,
        map: impl Into<Expression>,
        key: impl Into<Expression>,
        then: Block,
        else_: Block,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            map: map.into().into(),
            key: key.into().into(),
            then: then.into(),
            else_: else_.into(),
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn map(&self) -> &Expression {
        &self.map
    }

    pub fn key(&self) -> &Expression {
        &self.key
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
