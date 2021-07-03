use super::expression::Expression;
use crate::{position::Position, types::Type};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    function: Arc<Expression>,
    arguments: Vec<Expression>,
    function_type: Option<Type>,
    position: Position,
}

impl Call {
    // TODO Move function type to top.
    pub fn new(
        function: impl Into<Expression>,
        arguments: Vec<Expression>,
        function_type: Option<Type>,
        position: Position,
    ) -> Self {
        Self {
            function: Arc::new(function.into()),
            arguments,
            function_type,
            position,
        }
    }

    pub fn function(&self) -> &Expression {
        &self.function
    }

    pub fn arguments(&self) -> &[Expression] {
        &self.arguments
    }

    pub fn function_type(&self) -> Option<&Type> {
        self.function_type.as_ref()
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
