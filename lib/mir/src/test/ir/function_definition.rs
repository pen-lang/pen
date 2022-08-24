use crate::ir::*;

pub trait FunctionDefinitionFake {
    fn set_environment(&self, environment: Vec<Argument>) -> Self;
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
}
