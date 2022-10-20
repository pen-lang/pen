use super::expression::Expression;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    function: Rc<Expression>,
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
            function: Rc::new(function.into()),
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
