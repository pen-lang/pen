use crate::{ir::*, types::Type};

pub trait FunctionDefinitionFake {
    fn fake_with_environment(
        name: impl Into<String>,
        environment: Vec<Argument>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type>,
    ) -> Self;
}

impl FunctionDefinitionFake for FunctionDefinition {
    fn fake_with_environment(
        name: impl Into<String>,
        environment: Vec<Argument>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type>,
    ) -> Self {
        Self::with_options(name, environment, arguments, result_type, body, false)
    }
}
