use super::Expression;
use fnv::FnvHashSet;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct DiscardHeap {
    ids: FnvHashSet<String>,
    expression: Arc<Expression>,
}

impl DiscardHeap {
    pub fn new(ids: FnvHashSet<String>, expression: impl Into<Expression>) -> Self {
        Self {
            ids,
            expression: expression.into().into(),
        }
    }

    pub fn ids(&self) -> &FnvHashSet<String> {
        &self.ids
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
