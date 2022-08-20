use crate::ir::*;

pub struct Context {
    function_definitions: Vec<GlobalFunctionDefinition>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            function_definitions: vec![],
        }
    }

    pub fn add_function_definition(&mut self, definition: FunctionDefinition) -> String {
        let name = format!(
            "mir:lift:{}:{}",
            self.function_definitions.len(),
            definition.name()
        );

        self.function_definitions
            .push(GlobalFunctionDefinition::new(
                FunctionDefinition::with_options(
                    &name,
                    definition.environment().to_vec(),
                    definition.arguments().to_vec(),
                    definition.result_type().clone(),
                    Let::new(
                        definition.name(),
                        definition.type_().clone(),
                        Variable::new(&name),
                        definition.body().clone(),
                    ),
                    definition.is_thunk(),
                ),
                false,
            ));

        name
    }

    pub fn into_function_definitions(self) -> Vec<GlobalFunctionDefinition> {
        self.function_definitions
    }
}
