use super::{Argument, Block};
use crate::{position::Position, types::Type};

#[derive(Clone, Debug, PartialEq)]
pub struct Lambda {
    arguments: Vec<Argument>,
    body: Block,
    type_: Type,
    position: Position,
}

impl Lambda {
    pub fn new(
        arguments: Vec<Argument>,
        body: impl Into<Block>,
        type_: impl Into<Type>,
        position: Position,
    ) -> Self {
        Self {
            arguments,
            body: body.into(),
            type_: type_.into(),
            position,
        }
    }

    pub fn arguments(&self) -> &[Argument] {
        &self.arguments
    }

    pub fn body(&self) -> &Block {
        &self.body
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
