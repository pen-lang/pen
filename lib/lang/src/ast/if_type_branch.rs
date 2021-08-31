
use super::Block;
use crate::types::Type;

#[derive(Clone, Debug, PartialEq)]
pub struct IfTypeBranch {
    type_: Type,
    block: Block,
}

impl IfTypeBranch {
    pub fn new(type_: impl Into<Type>, block: Block) -> Self {
        Self {
            type_: type_.into(),
            block,
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn block(&self) -> &Block {
        &self.block
    }
}
