use super::expression::Expression;
use position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    function: Arc<Expression>,
    arguments: Vec<Expression>,
    position: Position,
}

impl Call {
    pub fn new(
        function: impl Into<Expression>,
        arguments: Vec<Expression>,
        position: Position,
    ) -> Self {
        Self {
            function: Arc::new(function.into()),
            arguments,
            position,
        }
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
