use super::{Expression, RecordUpdateField};
use crate::types::Type;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordUpdate {
    type_: Type,
    record: Arc<Expression>,
    fields: Vec<RecordUpdateField>,
}

impl RecordUpdate {
    pub fn new(
        type_: impl Into<Type>,
        record: impl Into<Expression>,
        fields: Vec<RecordUpdateField>,
    ) -> Self {
        Self {
            type_: type_.into(),
            record: record.into().into(),
            fields,
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn record(&self) -> &Expression {
        &self.record
    }

    pub fn fields(&self) -> &[RecordUpdateField] {
        &self.fields
    }
}
