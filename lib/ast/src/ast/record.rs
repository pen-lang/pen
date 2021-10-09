use super::{expression::Expression, RecordField};
use crate::types::Type;
use position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    type_: Type,
    record: Arc<Option<Expression>>,
    fields: Vec<RecordField>,
    position: Position,
}

impl Record {
    pub fn new(
        type_: impl Into<Type>,
        record: Option<Expression>,
        fields: Vec<RecordField>,
        position: Position,
    ) -> Self {
        Self {
            type_: type_.into(),
            record: Arc::new(record),
            fields,
            position,
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
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
