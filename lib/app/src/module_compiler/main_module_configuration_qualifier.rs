use crate::{application_configuration::MainModuleConfiguration, error::ApplicationError};
use fnv::FnvHashMap;
use std::error::Error;

pub fn qualify(
    configuration: &MainModuleConfiguration,
    context_interfaces: &FnvHashMap<String, interface::Module>,
) -> Result<hir_mir::MainModuleConfiguration, Box<dyn Error>> {
    Ok(hir_mir::MainModuleConfiguration {
        source_main_function_name: configuration.source_main_function_name.clone(),
        object_main_function_name: configuration.object_main_function_name.clone(),
        context_type_name: configuration.main_context_type_name.clone(),
        contexts: context_interfaces
            .iter()
            .map(|(key, interface)| Ok((key.clone(), qualify_context(interface, configuration)?)))
            .collect::<Result<_, Box<dyn Error>>>()?,
    })
}

fn qualify_context(
    context_interface: &interface::Module,
    configuration: &MainModuleConfiguration,
) -> Result<hir_mir::ContextConfiguration, Box<dyn Error>> {
    Ok(hir_mir::ContextConfiguration {
        context_type_name: context_interface
            .type_definitions()
            .iter()
            .find_map(|definition| {
                if definition.original_name() == configuration.system_context_type_name
                    && definition.is_public()
                {
                    Some(definition.name())
                } else {
                    None
                }
            })
            .into_iter()
            .chain(context_interface.type_aliases().iter().find_map(|alias| {
                if alias.original_name() == configuration.system_context_type_name
                    && alias.is_public()
                {
                    Some(alias.name())
                } else {
                    None
                }
            }))
            .next()
            .ok_or(ApplicationError::ContextTypeNotFound)?
            .into(),
        new_context_function_name: context_interface
            .declarations()
            .iter()
            .find(|definition| {
                definition.original_name() == configuration.new_system_context_function_name
            })
            .ok_or(ApplicationError::NewContextFunctionNotFound)?
            .name()
            .into(),
    })
}
