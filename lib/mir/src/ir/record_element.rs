use super::expression::Expression;
use crate::types;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordElement {
    type_: types::Record,
    index: usize,
    record: Arc<Expression>,
}

impl RecordElement {
    pub fn new(type_: types::Record, index: usize, record: impl Into<Expression>) -> Self {
        Self {
            type_,
            index,
            record: record.into().into(),
        }
    }

    pub fn type_(&self) -> &types::Record {
        &self.type_
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn record(&self) -> &Expression {
        &self.record
    }
}
