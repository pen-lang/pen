use super::{error::CompileError, main_module_configuration::MainModuleConfiguration};
use fnv::FnvHashMap;
use hir::{ir::*, types};

const MAIN_FUNCTION_WRAPPER_SUFFIX: &str = ":wrapper";

pub fn compile(
    module: &Module,
    main_module_configuration: &MainModuleConfiguration,
) -> Result<Module, CompileError> {
    let main_function_definition = module
        .function_definitions()
        .iter()
        .find(|definition| {
            definition.original_name() == main_module_configuration.source_main_function_name
        })
        .ok_or_else(|| CompileError::MainFunctionNotFound(module.position().clone()))?;
    let position = main_function_definition.position();

    let context_type_definition = TypeDefinition::new(
        &main_module_configuration.context_type_name,
        &main_module_configuration.context_type_name,
        main_module_configuration
            .contexts
            .iter()
            .map(|(key, configuration)| {
                Ok(types::RecordField::new(
                    key,
                    types::Reference::new(&configuration.context_type_name, position.clone()),
                ))
            })
            .collect::<Result<Vec<_>, CompileError>>()?,
        true,
        false,
        false,
        position.clone(),
    );
    let context_type = types::Record::new(
        context_type_definition.name(),
        context_type_definition.original_name(),
        position.clone(),
    );
    let new_context_function_declarations = main_module_configuration
        .contexts
        .iter()
        .map(|(key, configuration)| {
            Ok((
                key.clone(),
                module
                    .function_declarations()
                    .iter()
                    .find(|definition| definition.name() == configuration.new_context_function_name)
                    .ok_or_else(|| {
                        CompileError::NewContextFunctionNotFound(module.position().clone())
                    })?,
            ))
        })
        .collect::<Result<FnvHashMap<_, _>, CompileError>>()?;
    let function_type = types::Function::new(
        vec![context_type.clone().into()],
        types::None::new(position.clone()),
        position.clone(),
    );

    Ok(Module::new(
        module
            .type_definitions()
            .iter()
            .cloned()
            .chain([context_type_definition])
            .collect(),
        module.type_aliases().to_vec(),
        module.foreign_declarations().to_vec(),
        module.function_declarations().to_vec(),
        module
            .function_definitions()
            .iter()
            .cloned()
            .chain([FunctionDefinition::new(
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
                        vec![
                            RecordConstruction::new(
                                context_type,
                                new_context_function_declarations
                                    .iter()
                                    .map(|(key, definition)| {
                                        RecordField::new(
                                            key,
                                            Call::new(
                                                None,
                                                Variable::new(
                                                    definition.name(),
                                                    definition.position().clone(),
                                                ),
                                                vec![],
                                                position.clone(),
                                            ),
                                            position.clone(),
                                        )
                                    })
                                    .collect(),
                                position.clone(),
                            )
                            .into(),
                        ],
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
