use super::expression::Expression;
use crate::types;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordField(Rc<RecordFieldInner>);

#[derive(Debug, PartialEq)]
struct RecordFieldInner {
    type_: types::Record,
    index: usize,
    record: Expression,
}

impl RecordField {
    pub fn new(type_: types::Record, index: usize, record: impl Into<Expression>) -> Self {
        Self(
            RecordFieldInner {
                type_,
                index,
                record: record.into(),
            }
            .into(),
        )
    }

    pub fn type_(&self) -> &types::Record {
        &self.0.type_
    }

    pub fn index(&self) -> usize {
        self.0.index
    }

    pub fn record(&self) -> &Expression {
        &self.0.record
    }
}
