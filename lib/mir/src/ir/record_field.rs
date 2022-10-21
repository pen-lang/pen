use super::expression::Expression;
use crate::types;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordField {
    type_: types::Record,
    index: usize,
    record: Rc<Expression>,
}

impl RecordField {
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
