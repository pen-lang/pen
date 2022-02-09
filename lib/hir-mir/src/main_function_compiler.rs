use super::{error::CompileError, main_module_configuration::MainModuleConfiguration};
use hir::{ir::*, types};

const MAIN_FUNCTION_WRAPPER_SUFFIX: &str = "__wrapper";

pub fn compile(
    module: &Module,
    main_module_configuration: &MainModuleConfiguration,
) -> Result<Module, CompileError> {
    let definition = module
        .definitions()
        .iter()
        .find(|definition| {
            definition.original_name() == main_module_configuration.source_main_function_name
        })
        .ok_or_else(|| CompileError::MainFunctionNotFound(module.position().clone()))?;

    let position = definition.position();

    let context_type = module
        .type_aliases()
        .iter()
        .find(|alias| alias.name() == main_module_configuration.context_type_name)
        .ok_or_else(|| CompileError::ContextTypeUndefined(module.position().clone()))?
        .type_();
    let function_type = types::Function::new(
        vec![context_type.clone()],
        types::None::new(position.clone()),
        position.clone(),
    );
    let arguments = function_type
        .arguments()
        .iter()
        .enumerate()
        .map(|(index, type_)| Argument::new(format!("x{}", index), type_.clone()))
        .collect::<Vec<_>>();

    Ok(Module::new(
        module.type_definitions().to_vec(),
        module.type_aliases().to_vec(),
        module.foreign_declarations().to_vec(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .cloned()
            .chain([Definition::new(
                main_module_configuration
                    .object_main_function_name
                    .to_owned()
                    + MAIN_FUNCTION_WRAPPER_SUFFIX,
                &main_module_configuration.object_main_function_name,
                Lambda::new(
                    arguments.clone(),
                    function_type.result().clone(),
                    Call::new(
                        None,
                        Variable::new(definition.name(), position.clone()),
                        arguments
                            .iter()
                            .map(|argument| Variable::new(argument.name(), position.clone()).into())
                            .collect(),
                        position.clone(),
                    ),
                    position.clone(),
                ),
                ForeignDefinitionConfiguration::new(CallingConvention::Native).into(),
                false,
                position.clone(),
            )])
            .collect(),
        module.position().clone(),
    ))
}
