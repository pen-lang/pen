use super::{Expression, RecordUpdateField};
use crate::types;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordUpdate {
    type_: types::Record,
    record: Arc<Expression>,
    fields: Vec<RecordUpdateField>,
}

impl RecordUpdate {
    pub fn new(
        type_: types::Record,
        record: impl Into<Expression>,
        fields: Vec<RecordUpdateField>,
    ) -> Self {
        Self {
            type_,
            record: record.into().into(),
            fields,
        }
    }

    pub fn type_(&self) -> &types::Record {
        &self.type_
    }

    pub fn record(&self) -> &Expression {
        &self.record
    }

    pub fn fields(&self) -> &[RecordUpdateField] {
        &self.fields
    }
}
