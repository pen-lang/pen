use crate::{application_configuration::MainModuleConfiguration, error::ApplicationError};
use std::error::Error;

pub fn qualify(
    configuration: &MainModuleConfiguration,
    context_interface: &interface::Module,
) -> Result<MainModuleConfiguration, Box<dyn Error>> {
    Ok(MainModuleConfiguration {
        source_main_function_name: configuration.source_main_function_name.clone(),
        object_main_function_name: configuration.object_main_function_name.clone(),
        // TODO Support type aliases too.
        context_type_name: context_interface
            .type_definitions()
            .iter()
            .find(|definition| {
                definition.original_name() == configuration.context_type_name
                    && definition.is_public()
            })
            .ok_or(ApplicationError::ContextTypeNotFound)?
            .name()
            .into(),
        new_context_function_name: context_interface
            .declarations()
            .iter()
            .find(|definition| {
                definition.original_name() == configuration.new_context_function_name
            })
            .ok_or(ApplicationError::NewContextFunctionNotFound)?
            .name()
            .into(),
    })
}
