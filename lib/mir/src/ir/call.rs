use super::expression::Expression;
use crate::types;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    type_: types::Function,
    function: Rc<Expression>,
    arguments: Vec<Expression>,
}

impl Call {
    pub fn new(
        type_: types::Function,
        function: impl Into<Expression>,
        arguments: Vec<Expression>,
    ) -> Self {
        Self {
            type_,
            function: function.into().into(),
            arguments,
        }
    }

    pub fn type_(&self) -> &types::Function {
        &self.type_
    }

    pub fn function(&self) -> &Expression {
        &self.function
    }

    pub fn arguments(&self) -> &[Expression] {
        &self.arguments
    }
}
