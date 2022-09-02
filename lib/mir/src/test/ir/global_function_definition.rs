use crate::ir::*;

pub trait GlobalFunctionDefinitionFake {
    fn fake(definition: FunctionDefinition) -> Self;
}

impl GlobalFunctionDefinitionFake for GlobalFunctionDefinition {
    fn fake(definition: FunctionDefinition) -> Self {
        Self::new(definition, false)
    }
}
