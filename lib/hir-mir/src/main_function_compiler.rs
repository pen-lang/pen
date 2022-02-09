use super::{error::CompileError, main_module_configuration::MainModuleConfiguration};
use fnv::FnvHashMap;
use hir::{
    analysis::types::type_resolver,
    ir::*,
    types::{self, Type},
};

const MAIN_FUNCTION_WRAPPER_SUFFIX: &str = "__wrapper";

pub fn compile(
    module: &Module,
    types: &FnvHashMap<String, Type>,
    main_module_configuration: &MainModuleConfiguration,
) -> Result<Module, CompileError> {
    let main_function_definition = module
        .definitions()
        .iter()
        .find(|definition| {
            definition.original_name() == main_module_configuration.source_main_function_name
        })
        .ok_or_else(|| CompileError::MainFunctionNotFound(module.position().clone()))?;
    let position = main_function_definition.position();

    let context_type = type_resolver::resolve(
        &types::Reference::new(
            &main_module_configuration.context_type_name,
            position.clone(),
        ),
        types,
    )?;
    let function_type = types::Function::new(
        vec![context_type],
        types::None::new(position.clone()),
        position.clone(),
    );
    let new_context_function_definition = module
        .declarations()
        .iter()
        .find(|definition| definition.name() == main_module_configuration.new_context_function_name)
        .ok_or_else(|| CompileError::NewContextFunctionNotFound(module.position().clone()))?;

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
                    vec![],
                    function_type.result().clone(),
                    Call::new(
                        None,
                        Variable::new(main_function_definition.name(), position.clone()),
                        vec![Call::new(
                            None,
                            Variable::new(
                                new_context_function_definition.name(),
                                new_context_function_definition.position().clone(),
                            ),
                            vec![],
                            position.clone(),
                        )
                        .into()],
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
