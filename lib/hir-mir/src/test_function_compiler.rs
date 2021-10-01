use super::{error::CompileError, test_module_configuration::TestModuleConfiguration};
use crate::type_context::TypeContext;
use hir::{ir::*, types};
use std::collections::{hash_map::DefaultHasher, BTreeMap};
use std::hash::{Hash, Hasher};

const TEST_FUNCTION_WRAPPER_SUFFIX: &str = "__wrapper";

pub fn compile(
    module: &Module,
    type_context: &TypeContext,
    configuration: &TestModuleConfiguration,
) -> Result<(Module, BTreeMap<String, String>), CompileError> {
    let position = module.position();
    let context_type_definition = module
        .type_definitions()
        .iter()
        .find(|definition| definition.name() == configuration.context_type_name)
        .ok_or_else(|| CompileError::TestContextTypeUndefined(position.clone()))?;
    let arguments = vec![Argument::new(
        "ctx",
        types::Record::new(
            context_type_definition.name(),
            context_type_definition.position().clone(),
        )
        .clone(),
    )];

    let definitions = module
        .definitions()
        .iter()
        .filter(|definition| definition.is_public())
        .collect::<Vec<_>>();

    Ok((
        Module::new(
            module.type_definitions().to_vec(),
            module.type_aliases().to_vec(),
            module.foreign_declarations().to_vec(),
            module.declarations().to_vec(),
            module
                .definitions()
                .iter()
                .cloned()
                .chain(definitions.iter().map(|definition| {
                    let position = definition.position();

                    Definition::new(
                        definition.name().to_owned() + TEST_FUNCTION_WRAPPER_SUFFIX,
                        compile_test_name(definition.name(), configuration),
                        Lambda::new(
                            arguments.clone(),
                            types::Union::new(
                                types::None::new(position.clone()),
                                types::Record::new(
                                    &type_context.error_type_configuration().error_type_name,
                                    position.clone(),
                                )
                                .clone(),
                                position.clone(),
                            ),
                            Call::new(
                                None,
                                Variable::new(definition.name(), position.clone()),
                                arguments
                                    .iter()
                                    .map(|argument| {
                                        Variable::new(argument.name(), position.clone()).into()
                                    })
                                    .collect(),
                                position.clone(),
                            ),
                            position.clone(),
                        ),
                        ForeignDefinitionConfiguration::new(CallingConvention::C).into(),
                        false,
                        position.clone(),
                    )
                }))
                .collect(),
            position.clone(),
        ),
        definitions
            .iter()
            .map(|definition| {
                (
                    compile_test_name(definition.name(), configuration),
                    definition.original_name().into(),
                )
            })
            .collect(),
    ))
}

fn compile_test_name(name: &str, configuration: &TestModuleConfiguration) -> String {
    let mut hasher = DefaultHasher::new();

    name.hash(&mut hasher);

    configuration.test_function_prefix.to_owned() + &format!("{:x}", hasher.finish())
}
