use super::{expression::Expression, Block};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct IfBranch {
    condition: Arc<Expression>,
    block: Arc<Block>,
}

impl IfBranch {
    pub fn new(condition: impl Into<Expression>, block: Block) -> Self {
        Self {
            condition: condition.into().into(),
            block: block.into(),
        }
    }

    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    pub fn block(&self) -> &Block {
        &self.block
    }
}
