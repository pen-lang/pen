use super::expression::Expression;
use crate::types;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Call(Arc<CallInner>);

#[derive(Debug, PartialEq)]
struct CallInner {
    type_: types::Function,
    function: Expression,
    arguments: Vec<Expression>,
}

impl Call {
    pub fn new(
        type_: types::Function,
        function: impl Into<Expression>,
        arguments: Vec<Expression>,
    ) -> Self {
        Self(
            CallInner {
                type_,
                function: function.into(),
                arguments,
            }
            .into(),
        )
    }

    pub fn type_(&self) -> &types::Function {
        &self.0.type_
    }

    pub fn function(&self) -> &Expression {
        &self.0.function
    }

    pub fn arguments(&self) -> &[Expression] {
        &self.0.arguments
    }
}
