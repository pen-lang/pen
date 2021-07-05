use super::{error::CompileError, main_module_configuration::MainModuleConfiguration};
use crate::{
    hir::*,
    types::{analysis::type_resolver, Type},
};
use std::collections::HashMap;

const MAIN_MODULE_PREFIX: &str = "main:";

pub fn compile(
    module: &Module,
    types: &HashMap<String, Type>,
    main_module_configuration: &MainModuleConfiguration,
) -> Result<Module, CompileError> {
    let position = module
        .definitions()
        .iter()
        .find(|definition| definition.name() == main_module_configuration.source_main_function_name)
        .ok_or_else(|| CompileError::MainFunctionNotFound(module.position().clone()))?
        .position();

    let type_ = module
        .type_aliases()
        .iter()
        .find(|alias| alias.name() == main_module_configuration.main_function_type_name)
        .ok_or_else(|| CompileError::MainFunctionTypeUndefined(module.position().clone()))?
        .type_();
    let function_type = type_resolver::resolve_function(type_, types)?
        .ok_or_else(|| CompileError::FunctionExpected(type_.position().clone()))?;
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
            .chain(vec![Definition::new(
                MAIN_MODULE_PREFIX.to_owned()
                    + &main_module_configuration.object_main_function_name,
                &main_module_configuration.object_main_function_name,
                Lambda::new(
                    arguments.clone(),
                    function_type.result().clone(),
                    Call::new(
                        Variable::new(
                            &main_module_configuration.source_main_function_name,
                            position.clone(),
                        ),
                        arguments
                            .iter()
                            .map(|argument| Variable::new(argument.name(), position.clone()).into())
                            .collect(),
                        None,
                        position.clone(),
                    ),
                    position.clone(),
                ),
                true,
                false,
                position.clone(),
            )])
            .collect(),
        module.position().clone(),
    ))
}
