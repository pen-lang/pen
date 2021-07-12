use super::expression::Expression;
use crate::{position::Position, types::Type};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    function_type: Option<Type>,
    function: Arc<Expression>,
    arguments: Vec<Expression>,
    position: Position,
}

impl Call {
    pub fn new(
        function_type: Option<Type>,
        function: impl Into<Expression>,
        arguments: Vec<Expression>,
        position: Position,
    ) -> Self {
        Self {
            function_type,
            function: Arc::new(function.into()),
            arguments,
            position,
        }
    }

    pub fn function_type(&self) -> Option<&Type> {
        self.function_type.as_ref()
    }

    pub fn function(&self) -> &Expression {
        &self.function
    }

    pub fn arguments(&self) -> &[Expression] {
        &self.arguments
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
