use super::{expression::Expression, Block};
use crate::{position::Position, types::Type};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct IfList {
    argument: Arc<Expression>,
    type_: Type,
    first_name: String,
    rest_name: String,
    then: Arc<Block>,
    else_: Arc<Block>,
    position: Position,
}

impl IfList {
    pub fn new(
        argument: impl Into<Expression>,
        type_: impl Into<Type>,
        first_name: impl Into<String>,
        rest_name: impl Into<String>,
        then: Block,
        else_: Block,
        position: Position,
    ) -> Self {
        Self {
            argument: Arc::new(argument.into()),
            type_: type_.into(),
            first_name: first_name.into(),
            rest_name: rest_name.into(),
            then: then.into(),
            else_: else_.into(),
            position,
        }
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub fn type_(&self) -> &Type {
        &self.type_
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
