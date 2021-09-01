use super::{expression::Expression, RecordElement};
use crate::types::Type;
use position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordUpdate {
    type_: Type,
    record: Arc<Expression>,
    elements: Vec<RecordElement>,
    position: Position,
}

impl RecordUpdate {
    pub fn new(
        type_: impl Into<Type>,
        record: impl Into<Expression>,
        elements: Vec<RecordElement>,
        position: Position,
    ) -> Self {
        Self {
            type_: type_.into(),
            record: Arc::new(record.into()),
            elements,
            position,
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn record(&self) -> &Expression {
        &self.record
    }

    pub fn elements(&self) -> &[RecordElement] {
        &self.elements
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
