use crate::{ir::*, types::Type};

pub trait FunctionDefinitionFake {
    fn set_environment(&self, environment: Vec<Argument>) -> Self;

    fn fake_with_environment(
        name: impl Into<String>,
        environment: Vec<Argument>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type>,
    ) -> Self;
}

impl FunctionDefinitionFake for FunctionDefinition {
    fn set_environment(&self, environment: Vec<Argument>) -> Self {
        Self::with_options(
            self.name(),
            environment,
            self.arguments().to_vec(),
            self.result_type().clone(),
            self.body().clone(),
            self.is_thunk(),
        )
    }

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
