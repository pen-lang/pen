use super::expression::Expression;
use crate::{position::Position, types::Type};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    type_: Type,
    function: Arc<Expression>,
    arguments: Vec<Expression>,
    position: Position,
}

impl Call {
    pub fn new(
        type_: impl Into<Type>,
        function: impl Into<Expression>,
        arguments: Vec<Expression>,
        position: Position,
    ) -> Self {
        Self {
            type_: type_.into(),
            function: Arc::new(function.into()),
            arguments,
            position,
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
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
