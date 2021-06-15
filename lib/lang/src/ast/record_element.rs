use super::expression::Expression;
use crate::position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordElement {
    argument: Arc<Expression>,
    element_name: String,
    position: Position,
}

impl RecordElement {
    pub fn new(
        argument: impl Into<Expression>,
        element_name: impl Into<String>,
        position: Position,
    ) -> Self {
        Self {
            argument: argument.into().into(),
            element_name: element_name.into(),
            position,
        }
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub fn element_name(&self) -> &str {
        &self.element_name
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
