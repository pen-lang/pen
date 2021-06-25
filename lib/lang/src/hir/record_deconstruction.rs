use super::expression::Expression;
use crate::{position::Position, types::Type};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordDeconstruction {
    type_: Option<Type>,
    element_name: String,
    record: Arc<Expression>,
    position: Position,
}

impl RecordDeconstruction {
    pub fn new(
        type_: Option<Type>,
        record: impl Into<Expression>,
        element_name: impl Into<String>,
        position: Position,
    ) -> Self {
        Self {
            type_,
            element_name: element_name.into(),
            record: Arc::new(record.into()),
            position,
        }
    }

    pub fn type_(&self) -> Option<&Type> {
        self.type_.as_ref()
    }

    pub fn element_name(&self) -> &str {
        &self.element_name
    }

    pub fn record(&self) -> &Expression {
        &self.record
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
