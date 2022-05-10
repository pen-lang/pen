use super::Expression;
use fnv::FnvHashSet;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct DiscardHeap {
    blocks: FnvHashSet<String>,
    expression: Arc<Expression>,
}

impl DiscardHeap {
    pub fn new(blocks: FnvHashSet<String>, expression: impl Into<Expression>) -> Self {
        Self {
            blocks,
            expression: expression.into().into(),
        }
    }

    pub fn blocks(&self) -> &FnvHashSet<String> {
        &self.blocks
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
