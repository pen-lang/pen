use super::expression::Expression;
use crate::types::Type;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordDeconstruction {
    type_: Option<Type>,
    field_name: String,
    record: Rc<Expression>,
    position: Position,
}

impl RecordDeconstruction {
    pub fn new(
        type_: Option<Type>,
        record: impl Into<Expression>,
        field_name: impl Into<String>,
        position: Position,
    ) -> Self {
        Self {
            type_,
            field_name: field_name.into(),
            record: Rc::new(record.into()),
            position,
        }
    }

    pub fn type_(&self) -> Option<&Type> {
        self.type_.as_ref()
    }

    pub fn field_name(&self) -> &str {
        &self.field_name
    }

    pub fn record(&self) -> &Expression {
        &self.record
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
