mod compile_configuration;
mod concurrency_configuration;
mod context;
mod downcast_compiler;
mod duplicate_function_name_validator;
mod duplicate_type_name_validator;
mod environment_creator;
mod error;
mod error_type_configuration;
mod expression_compiler;
mod generic_type_definition_compiler;
mod list_type_configuration;
mod main_function_compiler;
mod main_module_configuration;
mod module_compiler;
mod module_interface_compiler;
mod record_field_validator;
mod spawn_function_declaration_compiler;
mod string_type_configuration;
mod test_function_compiler;
mod test_module_configuration;
mod transformation;
mod try_operation_validator;
mod type_checker;
mod type_coercer;
mod type_compiler;
mod type_extractor;
mod type_inferrer;

use self::{context::CompileContext, transformation::record_equal_function_transformer};
pub use compile_configuration::CompileConfiguration;
pub use concurrency_configuration::ConcurrencyConfiguration;
pub use error::CompileError;
pub use error_type_configuration::ErrorTypeConfiguration;
use hir::{analysis::types::type_existence_validator, ir::*};
pub use list_type_configuration::ListTypeConfiguration;
pub use main_module_configuration::*;
pub use string_type_configuration::StringTypeConfiguration;
pub use test_module_configuration::TestModuleConfiguration;

pub fn compile_main(
    module: &Module,
    compile_configuration: &CompileConfiguration,
    main_module_configuration: &MainModuleConfiguration,
) -> Result<mir::ir::Module, CompileError> {
    let context = CompileContext::new(module, compile_configuration.clone().into());
    let module =
        main_function_compiler::compile(module, context.types(), main_module_configuration)?;
    let (module, _) = compile_module(
        &module,
        &CompileContext::new(&module, compile_configuration.clone().into()),
    )?;

    Ok(module)
}

pub fn compile(
    module: &Module,
    compile_configuration: &CompileConfiguration,
) -> Result<(mir::ir::Module, interface::Module), CompileError> {
    compile_module(
        module,
        &CompileContext::new(module, compile_configuration.clone().into()),
    )
}

pub fn compile_prelude(
    module: &Module,
) -> Result<(mir::ir::Module, interface::Module), CompileError> {
    compile_module(module, &CompileContext::new(module, None))
}

pub fn compile_test(
    module: &Module,
    compile_configuration: &CompileConfiguration,
    test_module_configuration: &TestModuleConfiguration,
) -> Result<(mir::ir::Module, test_info::Module), CompileError> {
    let context = CompileContext::new(module, compile_configuration.clone().into());

    let (module, test_information) =
        test_function_compiler::compile(module, &context, test_module_configuration)?;
    let (module, _) = compile_module(&module, &context)?;

    Ok((module, test_information))
}

fn compile_module(
    module: &Module,
    context: &CompileContext,
) -> Result<(mir::ir::Module, interface::Module), CompileError> {
    duplicate_function_name_validator::validate(module)?;
    duplicate_type_name_validator::validate(module)?;
    type_existence_validator::validate(
        module,
        &context.types().keys().cloned().collect(),
        &context.records().keys().cloned().collect(),
    )?;

    let module = record_equal_function_transformer::transform(module, context)?;
    let module = type_inferrer::infer_types(&module, context)?;
    type_checker::check_types(&module, context)?;
    try_operation_validator::validate(&module, context)?;
    record_field_validator::validate(&module, context)?;
    let module = type_coercer::coerce_types(&module, context)?;
    type_checker::check_types(&module, context)?;

    Ok((
        {
            let module = module_compiler::compile(&module, context)?;
            mir::analysis::check_types(&module)?;
            module
        },
        module_interface_compiler::compile(&module)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        compile_configuration::COMPILE_CONFIGURATION,
        error_type_configuration::ERROR_TYPE_CONFIGURATION,
    };
    use hir::{
        test::{DefinitionFake, ModuleFake, TypeDefinitionFake},
        types,
    };
    use position::{test::PositionFake, Position};

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
        compile_module(&Module::empty().set_definitions(vec![Definition::fake(
            "x",
            Lambda::new(
                vec![],
                types::Boolean::new(Position::fake()),
                Boolean::new(false, Position::fake()),
                Position::fake(),
            ),
            false,
        )]))?;

        Ok(())
    }

    #[test]
    fn compile_none() -> Result<(), CompileError> {
        compile_module(&Module::empty().set_definitions(vec![Definition::fake(
            "x",
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                None::new(Position::fake()),
                Position::fake(),
            ),
            false,
        )]))?;

        Ok(())
    }

    #[test]
    fn compile_number() -> Result<(), CompileError> {
        compile_module(&Module::empty().set_definitions(vec![Definition::fake(
            "x",
            Lambda::new(
                vec![],
                types::Number::new(Position::fake()),
                Number::new(42.0, Position::fake()),
                Position::fake(),
            ),
            false,
        )]))?;

        Ok(())
    }

    #[test]
    fn compile_string() -> Result<(), CompileError> {
        compile_module(&Module::empty().set_definitions(vec![Definition::fake(
            "x",
            Lambda::new(
                vec![],
                types::ByteString::new(Position::fake()),
                ByteString::new("foo", Position::fake()),
                Position::fake(),
            ),
            false,
        )]))?;

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
                .set_definitions(vec![Definition::fake(
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
                .set_definitions(vec![Definition::fake(
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
        let definition = Definition::fake(
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
            compile_module(&Module::empty().set_definitions(vec![definition.clone(), definition])),
            Err(CompileError::DuplicateFunctionNames(
                Position::fake(),
                Position::fake()
            ))
        );
    }

    #[test]
    fn fail_to_compile_invalid_try_operator_in_function() {
        assert_eq!(
            compile_module(
                &Module::empty()
                    .set_type_definitions(vec![TypeDefinition::fake(
                        "error",
                        vec![],
                        false,
                        false,
                        false
                    )])
                    .set_definitions(vec![Definition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new(
                                "x",
                                types::Union::new(
                                    types::None::new(Position::fake()),
                                    types::Reference::new(
                                        &ERROR_TYPE_CONFIGURATION.error_type_name,
                                        Position::fake(),
                                    ),
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
                    )])
            ),
            Err(CompileError::InvalidTryOperation(Position::fake()))
        );
    }
}
