use super::{Expression, RecordField};
use std::{ops::Deref, sync::Arc};

#[derive(Clone, Debug, PartialEq)]
pub struct BorrowRecordField {
    field: RecordField,
    record_name: String,
    field_name: String,
    expression: Arc<Expression>,
}

impl BorrowRecordField {
    pub fn new(
        field: RecordField,
        record_name: impl Into<String>,
        field_name: impl Into<String>,
        expression: impl Into<Expression>,
    ) -> Self {
        Self {
            field,
            record_name: record_name.into(),
            field_name: field_name.into(),
            expression: expression.into().into(),
        }
    }

    pub fn field(&self) -> &RecordField {
        &self.field
    }

    pub fn record_name(&self) -> &str {
        &self.record_name
    }

    pub fn field_name(&self) -> &str {
        &self.field_name
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}

impl Deref for BorrowRecordField {
    type Target = RecordField;

    fn deref(&self) -> &RecordField {
        &self.field
    }
}
