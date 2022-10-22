use super::{Expression, RecordUpdateField};
use crate::types;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordUpdate(Arc<RecordUpdateInner>);

#[derive(Debug, PartialEq)]
struct RecordUpdateInner {
    type_: types::Record,
    record: Expression,
    fields: Vec<RecordUpdateField>,
}

impl RecordUpdate {
    pub fn new(
        type_: types::Record,
        record: impl Into<Expression>,
        fields: Vec<RecordUpdateField>,
    ) -> Self {
        Self(
            RecordUpdateInner {
                type_,
                record: record.into(),
                fields,
            }
            .into(),
        )
    }

    pub fn type_(&self) -> &types::Record {
        &self.0.type_
    }

    pub fn record(&self) -> &Expression {
        &self.0.record
    }

    pub fn fields(&self) -> &[RecordUpdateField] {
        &self.0.fields
    }
}
