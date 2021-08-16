use crate::application_configuration::MainModuleConfiguration;
use crate::error::ApplicationError;
use std::error::Error;

pub fn qualify(
    configuration: &MainModuleConfiguration,
    main_function_interface: &lang::interface::Module,
) -> Result<MainModuleConfiguration, Box<dyn Error>> {
    Ok(MainModuleConfiguration {
        source_main_function_name: configuration.source_main_function_name.clone(),
        object_main_function_name: configuration.object_main_function_name.clone(),
        main_function_type_name: main_function_interface
            .type_aliases()
            .iter()
            .find(|alias| {
                alias.original_name() == configuration.main_function_type_name && alias.is_public()
            })
            .ok_or(ApplicationError::MainFunctionTypeNotFound)?
            .name()
            .into(),
    })
}
