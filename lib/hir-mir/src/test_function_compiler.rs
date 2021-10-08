use super::{error::CompileError, test_module_configuration::TestModuleConfiguration};
use crate::type_context::TypeContext;
use hir::{ir::*, types};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

const TEST_FUNCTION_WRAPPER_SUFFIX: &str = "__wrapper";
const TEST_RESULT_TYPE_NAME: &str = "_testResult";

pub fn compile(
    module: &Module,
    type_context: &TypeContext,
    configuration: &TestModuleConfiguration,
) -> Result<(Module, test_info::Module), CompileError> {
    let position = module.position();

    let definitions = module
        .definitions()
        .iter()
        .filter(|definition| definition.is_public())
        .collect::<Vec<_>>();

    Ok((
        Module::new(
            module
                .type_definitions()
                .iter()
                .cloned()
                .chain(vec![TypeDefinition::new(
                    TEST_RESULT_TYPE_NAME,
                    TEST_RESULT_TYPE_NAME,
                    vec![types::RecordElement::new(
                        "error",
                        types::Union::new(
                            types::None::new(position.clone()),
                            types::Record::new(
                                &type_context.error_type_configuration().error_type_name,
                                position.clone(),
                            ),
                            position.clone(),
                        ),
                    )],
                    false,
                    false,
                    false,
                    position.clone(),
                )])
                .collect(),
            module.type_aliases().to_vec(),
            module.foreign_declarations().to_vec(),
            module.declarations().to_vec(),
            module
                .definitions()
                .iter()
                .cloned()
                .chain(definitions.iter().map(|definition| {
                    let position = definition.position();
                    let result_type = types::Record::new(TEST_RESULT_TYPE_NAME, position.clone());

                    Definition::new(
                        definition.name().to_owned() + TEST_FUNCTION_WRAPPER_SUFFIX,
                        compile_test_name(definition.name(), configuration),
                        Lambda::new(
                            vec![],
                            result_type.clone(),
                            RecordConstruction::new(
                                result_type,
                                vec![RecordElement::new(
                                    "error",
                                    Call::new(
                                        None,
                                        Variable::new(definition.name(), position.clone()),
                                        vec![],
                                        position.clone(),
                                    ),
                                    position.clone(),
                                )],
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
        test_info::Module::new(
            module.position().path(),
            definitions
                .iter()
                .map(|definition| {
                    (
                        compile_test_name(definition.name(), configuration),
                        test_info::Function::new(
                            definition.original_name(),
                            definition.position().clone(),
                        ),
                    )
                })
                .collect(),
        ),
    ))
}

fn compile_test_name(name: &str, configuration: &TestModuleConfiguration) -> String {
    let mut hasher = DefaultHasher::new();

    name.hash(&mut hasher);

    configuration.test_function_prefix.to_owned() + &format!("{:x}", hasher.finish())
}
