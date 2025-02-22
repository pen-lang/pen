use super::{error::CompileError, test_module_configuration::TestModuleConfiguration};
use hir::{ir::*, types};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

const TEST_FUNCTION_WRAPPER_SUFFIX: &str = "__wrapper";
const RESULT_VARIABLE_NAME: &str = "$result";
const MESSAGE_VARIABLE_NAME: &str = "$message";
const NON_STRING_TEST_ERROR_MESSAGE: &str = "<non-string test error>";

pub fn compile(
    module: &Module,
    configuration: &TestModuleConfiguration,
) -> Result<(Module, test_info::Module), CompileError> {
    let position = module.position();

    let definitions = module
        .function_definitions()
        .iter()
        .filter(|definition| definition.is_public())
        .collect::<Vec<_>>();

    Ok((
        Module::new(
            module.type_definitions().to_vec(),
            module.type_aliases().to_vec(),
            module.foreign_declarations().to_vec(),
            module.function_declarations().to_vec(),
            module
                .function_definitions()
                .iter()
                .cloned()
                .chain(
                    definitions
                        .iter()
                        .map(|definition| {
                            let position = definition.position();

                            Ok(FunctionDefinition::new(
                                definition.name().to_owned() + TEST_FUNCTION_WRAPPER_SUFFIX,
                                compile_foreign_name(definition.name(), configuration),
                                Lambda::new(
                                    vec![],
                                    types::ByteString::new(position.clone()),
                                    IfType::new(
                                        "$result",
                                        Call::new(
                                            None,
                                            Variable::new(definition.name(), position.clone()),
                                            vec![],
                                            position.clone(),
                                        ),
                                        vec![IfTypeBranch::new(
                                            types::None::new(position.clone()),
                                            ByteString::new(vec![], position.clone()),
                                        )],
                                        Some(ElseBranch::new(
                                            None,
                                            IfType::new(
                                                MESSAGE_VARIABLE_NAME,
                                                Call::new(
                                                    None,
                                                    BuiltInFunction::new(
                                                        BuiltInFunctionName::Source,
                                                        position.clone(),
                                                    ),
                                                    vec![
                                                        Variable::new(
                                                            RESULT_VARIABLE_NAME,
                                                            position.clone(),
                                                        )
                                                        .into(),
                                                    ],
                                                    position.clone(),
                                                ),
                                                vec![IfTypeBranch::new(
                                                    types::ByteString::new(position.clone()),
                                                    Variable::new(
                                                        MESSAGE_VARIABLE_NAME,
                                                        position.clone(),
                                                    ),
                                                )],
                                                Some(ElseBranch::new(
                                                    None,
                                                    ByteString::new(
                                                        NON_STRING_TEST_ERROR_MESSAGE,
                                                        position.clone(),
                                                    ),
                                                    position.clone(),
                                                )),
                                                position.clone(),
                                            ),
                                            position.clone(),
                                        )),
                                        position.clone(),
                                    ),
                                    position.clone(),
                                ),
                                ForeignDefinitionConfiguration::new(CallingConvention::C).into(),
                                false,
                                position.clone(),
                            ))
                        })
                        .collect::<Result<Vec<_>, CompileError>>()?,
                )
                .collect(),
            position.clone(),
        ),
        test_info::Module::new(
            module.position().path(),
            definitions
                .iter()
                .map(|definition| {
                    test_info::Function::new(
                        definition.original_name(),
                        compile_foreign_name(definition.name(), configuration),
                        definition.position().clone(),
                    )
                })
                .collect(),
        ),
    ))
}

fn compile_foreign_name(name: &str, configuration: &TestModuleConfiguration) -> String {
    let mut hasher = DefaultHasher::new();

    name.hash(&mut hasher);

    configuration.test_function_prefix.to_owned() + &format!("{:x}", hasher.finish())
}
