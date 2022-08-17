mod built_in_call;
mod compile_configuration;
mod context;
mod downcast;
mod error;
mod error_type_configuration;
mod expression;
mod generic_type_definition;
mod list_type_configuration;
mod main_function;
mod main_module_configuration;
mod map_type_configuration;
mod module;
mod module_interface;
mod runtime_function_declaration;
mod string_type_configuration;
mod test_function;
mod test_module_configuration;
mod transformation;
mod type_;

pub use compile_configuration::CompileConfiguration;
use context::CompileContext;
pub use error::CompileError;
pub use error_type_configuration::ErrorTypeConfiguration;
use hir::ir::*;
pub use list_type_configuration::ListTypeConfiguration;
pub use main_module_configuration::*;
pub use map_type_configuration::{
    HashConfiguration, MapTypeConfiguration, MapTypeIterationConfiguration,
};
pub use string_type_configuration::StringTypeConfiguration;
pub use test_module_configuration::TestModuleConfiguration;
use transformation::{record_equal_function, record_hash_function};

pub fn compile_main(
    module: &Module,
    compile_configuration: &CompileConfiguration,
    main_module_configuration: &MainModuleConfiguration,
) -> Result<mir::ir::Module, CompileError> {
    let module = main_function::compile(module, main_module_configuration)?;
    let (module, _) = compile_module(&module, Some(compile_configuration))?;

    Ok(module)
}

pub fn compile(
    module: &Module,
    configuration: &CompileConfiguration,
) -> Result<(mir::ir::Module, interface::Module), CompileError> {
    compile_module(module, Some(configuration))
}

pub fn compile_prelude(
    module: &Module,
) -> Result<(mir::ir::Module, interface::Module), CompileError> {
    compile_module(module, None)
}

pub fn compile_test(
    module: &Module,
    compile_configuration: &CompileConfiguration,
    test_module_configuration: &TestModuleConfiguration,
) -> Result<(mir::ir::Module, test_info::Module), CompileError> {
    let (module, test_information) = test_function::compile(module, test_module_configuration)?;
    let (module, _) = compile_module(&module, Some(compile_configuration))?;

    Ok((module, test_information))
}

fn compile_module(
    module: &Module,
    configuration: Option<&CompileConfiguration>,
) -> Result<(mir::ir::Module, interface::Module), CompileError> {
    let context = CompileContext::new(module, configuration.cloned());

    let module = hir::analysis::analyze(context.analysis(), module)?;
    let module = record_equal_function::transform(&context, &module)?;
    let module = record_hash_function::transform(&context, &module)?;

    Ok((
        {
            let module = module::compile(&context, &module)?;
            mir::analysis::type_check::check(&module)?;
            module
        },
        module_interface::compile(&module)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        compile_configuration::COMPILE_CONFIGURATION, map_type_configuration::HASH_CONFIGURATION,
    };
    use hir::{
        analysis::AnalysisError,
        test::{FunctionDefinitionFake, ModuleFake, TypeDefinitionFake},
        types,
    };
    use once_cell::sync::Lazy;
    use position::{test::PositionFake, Position};

    static COMBINE_HASH_FUNCTION_DECLARATION: Lazy<FunctionDeclaration> = Lazy::new(|| {
        FunctionDeclaration::new(
            &HASH_CONFIGURATION.combine_function_name,
            types::Function::new(
                vec![
                    types::Number::new(Position::fake()).into(),
                    types::Number::new(Position::fake()).into(),
                ],
                types::Number::new(Position::fake()),
                Position::fake(),
            ),
            Position::fake(),
        )
    });

    // TODO Test types included in prelude modules by mocking them.
    fn compile_module(
        module: &Module,
    ) -> Result<(mir::ir::Module, interface::Module), CompileError> {
        compile(module, &COMPILE_CONFIGURATION)
    }

    #[test]
    fn compile_empty_module() -> Result<(), CompileError> {
        compile_module(&Module::empty())?;

        Ok(())
    }

    #[test]
    fn compile_boolean() -> Result<(), CompileError> {
        compile_module(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::Boolean::new(Position::fake()),
                    Boolean::new(false, Position::fake()),
                    Position::fake(),
                ),
                false,
            )]),
        )?;

        Ok(())
    }

    #[test]
    fn compile_none() -> Result<(), CompileError> {
        compile_module(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    None::new(Position::fake()),
                    Position::fake(),
                ),
                false,
            )]),
        )?;

        Ok(())
    }

    #[test]
    fn compile_number() -> Result<(), CompileError> {
        compile_module(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::Number::new(Position::fake()),
                    Number::new(42.0, Position::fake()),
                    Position::fake(),
                ),
                false,
            )]),
        )?;

        Ok(())
    }

    #[test]
    fn compile_string() -> Result<(), CompileError> {
        compile_module(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::ByteString::new(Position::fake()),
                    ByteString::new("foo", Position::fake()),
                    Position::fake(),
                ),
                false,
            )]),
        )?;

        Ok(())
    }

    #[test]
    fn compile_record_construction() {
        let reference_type = types::Reference::new("foo", Position::fake());

        compile_module(
            &Module::empty()
                .set_type_definitions(vec![TypeDefinition::fake(
                    "foo",
                    vec![types::RecordField::new(
                        "x",
                        types::None::new(Position::fake()),
                    )],
                    false,
                    false,
                    false,
                )])
                .set_function_declarations(vec![COMBINE_HASH_FUNCTION_DECLARATION.clone()])
                .set_function_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        reference_type.clone(),
                        RecordConstruction::new(
                            reference_type,
                            vec![RecordField::new(
                                "x",
                                None::new(Position::fake()),
                                Position::fake(),
                            )],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
        )
        .unwrap();
    }

    #[test]
    fn compile_record_deconstruction() {
        let reference_type = types::Reference::new("foo", Position::fake());

        compile_module(
            &Module::empty()
                .set_type_definitions(vec![TypeDefinition::fake(
                    "foo",
                    vec![types::RecordField::new(
                        "x",
                        types::None::new(Position::fake()),
                    )],
                    false,
                    false,
                    false,
                )])
                .set_function_declarations(vec![COMBINE_HASH_FUNCTION_DECLARATION.clone()])
                .set_function_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("r", reference_type)],
                        types::None::new(Position::fake()),
                        RecordDeconstruction::new(
                            None,
                            Variable::new("r", Position::fake()),
                            "x",
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
        )
        .unwrap();
    }

    #[test]
    fn fail_to_compile_duplicate_function_names() {
        let definition = FunctionDefinition::fake(
            "x",
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                None::new(Position::fake()),
                Position::fake(),
            ),
            false,
        );

        assert_eq!(
            compile_module(
                &Module::empty().set_function_definitions(vec![definition.clone(), definition])
            ),
            Err(AnalysisError::DuplicateFunctionNames(Position::fake(), Position::fake()).into())
        );
    }

    #[test]
    fn fail_to_compile_invalid_try_operator_in_function() {
        assert_eq!(
            compile_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new(
                            "x",
                            types::Union::new(
                                types::None::new(Position::fake()),
                                types::Error::new(Position::fake(),),
                                Position::fake(),
                            ),
                        )],
                        types::None::new(Position::fake()),
                        TryOperation::new(
                            None,
                            Variable::new("x", Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )
            ])),
            Err(AnalysisError::InvalidTryOperation(Position::fake()).into())
        );
    }
}
