use super::{expression::Expression, RecordField};
use position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    type_name: String,
    record: Arc<Option<Expression>>,
    fields: Vec<RecordField>,
    position: Position,
}

impl Record {
    pub fn new(
        type_name: impl Into<String>,
        record: Option<Expression>,
        fields: Vec<RecordField>,
        position: Position,
    ) -> Self {
        Self {
            type_name: type_name.into(),
            record: Arc::new(record),
            fields,
            position,
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn record(&self) -> Option<&Expression> {
        self.record.as_ref().as_ref()
    }

    pub fn fields(&self) -> &[RecordField] {
        &self.fields
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
