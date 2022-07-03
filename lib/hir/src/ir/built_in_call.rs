use super::expression::Expression;
use crate::types::Type;
use position::Position;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BuiltInFunction {
    Size,
    Spawn,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BuiltInCall {
    function_type: Option<Type>,
    function: BuiltInFunction,
    arguments: Vec<Expression>,
    position: Position,
}

impl BuiltInCall {
    pub fn new(
        function_type: Option<Type>,
        function: BuiltInFunction,
        arguments: Vec<Expression>,
        position: Position,
    ) -> Self {
        Self {
            function_type,
            function,
            arguments,
            position,
        }
    }

    pub fn function_type(&self) -> Option<&Type> {
        self.function_type.as_ref()
    }

    pub fn function(&self) -> BuiltInFunction {
        self.function
    }

    pub fn arguments(&self) -> &[Expression] {
        &self.arguments
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
