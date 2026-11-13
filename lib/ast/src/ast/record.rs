use super::{RecordField, expression::Expression};
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    type_name: String,
    record: Rc<Option<Expression>>,
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
            record: Rc::new(record),
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
