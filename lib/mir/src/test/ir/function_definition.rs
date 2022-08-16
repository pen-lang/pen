use crate::{ir::*, types::Type};

pub trait FunctionDefinitionFake {
    fn fake(
        name: impl Into<String>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type>,
    ) -> Self;

    fn fake_thunk(
        name: impl Into<String>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type>,
    ) -> Self;
}

impl FunctionDefinitionFake for FunctionDefinition {
    fn fake(
        name: impl Into<String>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type>,
    ) -> Self {
        Self::new(name, arguments, body, result_type, false)
    }

    fn fake_thunk(
        name: impl Into<String>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type>,
    ) -> Self {
        Self::thunk(name, body, result_type, false)
    }
}
