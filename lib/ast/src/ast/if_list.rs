use super::{Block, expression::Expression};
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct IfList {
    list: Rc<Expression>,
    first_name: String,
    rest_name: String,
    then: Rc<Block>,
    else_: Rc<Block>,
    position: Position,
}

impl IfList {
    pub fn new(
        list: impl Into<Expression>,
        first_name: impl Into<String>,
        rest_name: impl Into<String>,
        then: Block,
        else_: Block,
        position: Position,
    ) -> Self {
        Self {
            list: Rc::new(list.into()),
            first_name: first_name.into(),
            rest_name: rest_name.into(),
            then: then.into(),
            else_: else_.into(),
            position,
        }
    }

    pub fn list(&self) -> &Expression {
        &self.list
    }

    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn rest_name(&self) -> &str {
        &self.rest_name
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
