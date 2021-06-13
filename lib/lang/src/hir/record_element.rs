use super::expression::Expression;
use crate::{position::Position, types::Type};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordElement {
    type_: Type,
    element_name: String,
    argument: Arc<Expression>,
    position: Position,
}

impl RecordElement {
    pub fn new(
        type_: impl Into<Type>,
        element_name: impl Into<String>,
        argument: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            type_: type_.into(),
            element_name: element_name.into(),
            argument: Arc::new(argument.into()),
            position,
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn element_name(&self) -> &str {
        &self.element_name
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
