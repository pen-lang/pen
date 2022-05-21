use super::{Expression, RecordField};
use crate::types;
use std::ops::Deref;

#[derive(Clone, Debug, PartialEq)]
pub struct BorrowRecordField {
    record_field: RecordField,
}

impl BorrowRecordField {
    pub fn new(type_: types::Record, index: usize, record: impl Into<Expression>) -> Self {
        Self {
            record_field: RecordField::new(type_, index, record),
        }
    }
}

impl Deref for BorrowRecordField {
    type Target = RecordField;

    fn deref(&self) -> &RecordField {
        &self.record_field
    }
}
