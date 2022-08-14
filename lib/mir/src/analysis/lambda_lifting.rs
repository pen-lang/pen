use crate::ir::*;

struct Context {
    function_definitions: Vec<FunctionDefinition>,
}

pub fn transform(module: &Module) -> Module {
    let mut context = Context {
        function_definitions: vec![],
    };

    Module::new(
        module.type_definitions().to_vec(),
        module.foreign_declarations().to_vec(),
        module.foreign_definitions().to_vec(),
        module.function_declarations().to_vec(),
        module
            .function_definitions()
            .iter()
            .map(|definition| transform_function_definition(&mut context, definition))
            .collect(),
    )
}

fn transform_function_definition(
    context: &mut Context,
    definition: &FunctionDefinition,
) -> FunctionDefinition {
    todo!()
}
