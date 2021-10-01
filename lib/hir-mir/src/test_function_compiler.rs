use super::{error::CompileError, test_module_configuration::TestModuleConfiguration};
use crate::type_context::TypeContext;
use hir::{ir::*, types};
use std::{
    collections::{hash_map::DefaultHasher, BTreeMap},
    hash::{Hash, Hasher},
};

const TEST_FUNCTION_WRAPPER_SUFFIX: &str = "__wrapper";

pub fn compile(
    module: &Module,
    type_context: &TypeContext,
    configuration: &TestModuleConfiguration,
) -> Result<(Module, BTreeMap<String, String>), CompileError> {
    let position = module.position();

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
                            vec![],
                            types::Union::new(
                                types::None::new(position.clone()),
                                types::Record::new(
                                    &type_context.error_type_configuration().error_type_name,
                                    position.clone(),
                                ),
                                position.clone(),
                            ),
                            Call::new(
                                None,
                                Variable::new(definition.name(), position.clone()),
                                vec![],
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
