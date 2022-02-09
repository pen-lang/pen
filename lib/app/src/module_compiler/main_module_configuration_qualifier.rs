use crate::{application_configuration::MainModuleConfiguration, error::ApplicationError};
use std::error::Error;

pub fn qualify(
    configuration: &MainModuleConfiguration,
    context_interface: &interface::Module,
) -> Result<MainModuleConfiguration, Box<dyn Error>> {
    Ok(MainModuleConfiguration {
        source_main_function_name: configuration.source_main_function_name.clone(),
        object_main_function_name: configuration.object_main_function_name.clone(),
        context_type_name: context_interface
            .type_aliases()
            .iter()
            .find(|alias| {
                alias.original_name() == configuration.context_type_name && alias.is_public()
            })
            .ok_or(ApplicationError::ContextTypeNotFound)?
            .name()
            .into(),
    })
}
