use super::Block;
use crate::types::Type;

#[derive(Clone, Debug, PartialEq)]
pub struct Alternative {
    type_: Type,
    block: Block,
}

impl Alternative {
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
