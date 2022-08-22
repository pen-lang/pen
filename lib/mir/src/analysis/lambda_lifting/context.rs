use crate::ir::*;

#[derive(Debug)]
pub struct Context {
    name_index: usize,
    function_definitions: Vec<GlobalFunctionDefinition>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            name_index: 0,
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

    pub fn rename_free_variable(&mut self, name: &str) -> String {
        let name = format!("fv:{}:{}", name, self.name_index);

        self.name_index += 1;

        name
    }

    pub fn into_function_definitions(self) -> Vec<GlobalFunctionDefinition> {
        self.function_definitions
    }
}
