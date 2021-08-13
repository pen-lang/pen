mod dummy_type_configurations;
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
mod record_element_validator;
mod string_type_configuration;
mod transformation;
mod type_checker;
mod type_coercer;
mod type_compiler;
mod type_context;
mod type_extractor;
mod type_inferrer;

use self::{
    dummy_type_configurations::{
        DUMMY_ERROR_TYPE_CONFIGURATION, DUMMY_LIST_TYPE_CONFIGURATION,
        DUMMY_STRING_TYPE_CONFIGURATION,
    },
    transformation::record_equal_function_transformer,
    type_context::TypeContext,
};
use crate::{hir::*, interface};
pub use error::CompileError;
pub use error_type_configuration::ErrorTypeConfiguration;
pub use list_type_configuration::ListTypeConfiguration;
pub use main_module_configuration::MainModuleConfiguration;
pub use string_type_configuration::StringTypeConfiguration;

pub fn compile_main(
    module: &Module,
    list_type_configuration: &ListTypeConfiguration,
    string_type_configuration: &StringTypeConfiguration,
    error_type_configuration: &ErrorTypeConfiguration,
    main_module_configuration: &MainModuleConfiguration,
) -> Result<mir::ir::Module, CompileError> {
    let type_context = TypeContext::new(
        module,
        list_type_configuration,
        string_type_configuration,
        error_type_configuration,
    );
    let module =
        main_function_compiler::compile(module, type_context.types(), main_module_configuration)?;
    let (module, _) = compile_module(&module, &type_context)?;

    Ok(module)
}

pub fn compile(
    module: &Module,
    list_type_configuration: &ListTypeConfiguration,
    string_type_configuration: &StringTypeConfiguration,
    error_type_configuration: &ErrorTypeConfiguration,
) -> Result<(mir::ir::Module, interface::Module), CompileError> {
    compile_module(
        module,
        &TypeContext::new(
            module,
            list_type_configuration,
            string_type_configuration,
            error_type_configuration,
        ),
    )
}

pub fn compile_prelude(
    module: &Module,
) -> Result<(mir::ir::Module, interface::Module), CompileError> {
    compile_module(
        module,
        &TypeContext::new(
            module,
            &DUMMY_LIST_TYPE_CONFIGURATION,
            &DUMMY_STRING_TYPE_CONFIGURATION,
            &DUMMY_ERROR_TYPE_CONFIGURATION,
        ),
    )
}

fn compile_module(
    module: &Module,
    type_context: &TypeContext,
) -> Result<(mir::ir::Module, interface::Module), CompileError> {
    duplicate_function_name_validator::validate(module)?;
    duplicate_type_name_validator::validate(module)?;

    let module = record_equal_function_transformer::transform(module, type_context)?;
    let module = type_inferrer::infer_types(&module, type_context)?;
    type_checker::check_types(&module, type_context)?;
    record_element_validator::validate(&module, type_context)?;
    let module = type_coercer::coerce_types(&module, type_context)?;
    type_checker::check_types(&module, type_context)?;

    Ok((
        {
            let module = module_compiler::compile(&module, type_context)?;
            mir::analysis::check_types(&module)?;
            module
        },
        module_interface_compiler::compile(&module)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        error_type_configuration::ERROR_TYPE_CONFIGURATION,
        list_type_configuration::LIST_TYPE_CONFIGURATION, *,
    };
    use crate::{
        hir_mir::string_type_configuration::STRING_TYPE_CONFIGURATION, position::Position, types,
    };

    fn compile_module(
        module: &Module,
    ) -> Result<(mir::ir::Module, interface::Module), CompileError> {
        compile(
            module,
            &LIST_TYPE_CONFIGURATION,
            &STRING_TYPE_CONFIGURATION,
            &ERROR_TYPE_CONFIGURATION,
        )
    }

    #[test]
    fn compile_empty_module() -> Result<(), CompileError> {
        compile_module(&Module::empty())?;

        Ok(())
    }

    #[test]
    fn compile_boolean() -> Result<(), CompileError> {
        compile_module(
            &Module::empty().set_definitions(vec![Definition::without_source(
                "x",
                Lambda::new(
                    vec![],
                    types::Boolean::new(Position::dummy()),
                    Boolean::new(false, Position::dummy()),
                    Position::dummy(),
                ),
                false,
            )]),
        )?;

        Ok(())
    }

    #[test]
    fn compile_none() -> Result<(), CompileError> {
        compile_module(
            &Module::empty().set_definitions(vec![Definition::without_source(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::dummy()),
                    None::new(Position::dummy()),
                    Position::dummy(),
                ),
                false,
            )]),
        )?;

        Ok(())
    }

    #[test]
    fn compile_number() -> Result<(), CompileError> {
        compile_module(
            &Module::empty().set_definitions(vec![Definition::without_source(
                "x",
                Lambda::new(
                    vec![],
                    types::Number::new(Position::dummy()),
                    Number::new(42.0, Position::dummy()),
                    Position::dummy(),
                ),
                false,
            )]),
        )?;

        Ok(())
    }

    #[test]
    fn compile_string() -> Result<(), CompileError> {
        compile_module(
            &Module::empty().set_definitions(vec![Definition::without_source(
                "x",
                Lambda::new(
                    vec![],
                    types::ByteString::new(Position::dummy()),
                    ByteString::new("foo", Position::dummy()),
                    Position::dummy(),
                ),
                false,
            )]),
        )?;

        Ok(())
    }

    #[test]
    fn compile_record_construction() {
        let reference_type = types::Reference::new("foo", Position::dummy());

        compile_module(
            &Module::empty()
                .set_type_definitions(vec![TypeDefinition::without_source(
                    "foo",
                    vec![types::RecordElement::new(
                        "x",
                        types::None::new(Position::dummy()),
                    )],
                    false,
                    false,
                    false,
                )])
                .set_definitions(vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![],
                        reference_type.clone(),
                        RecordConstruction::new(
                            reference_type,
                            vec![RecordElement::new(
                                "x",
                                None::new(Position::dummy()),
                                Position::dummy(),
                            )],
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )]),
        )
        .unwrap();
    }

    #[test]
    fn compile_record_deconstruction() {
        let reference_type = types::Reference::new("foo", Position::dummy());

        compile_module(
            &Module::empty()
                .set_type_definitions(vec![TypeDefinition::without_source(
                    "foo",
                    vec![types::RecordElement::new(
                        "x",
                        types::None::new(Position::dummy()),
                    )],
                    false,
                    false,
                    false,
                )])
                .set_definitions(vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![Argument::new("r", reference_type.clone())],
                        types::None::new(Position::dummy()),
                        RecordDeconstruction::new(
                            None,
                            Variable::new("r", Position::dummy()),
                            "x",
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )]),
        )
        .unwrap();
    }

    #[test]
    fn fail_to_compile_duplicate_function_names() {
        let definition = Definition::without_source(
            "x",
            Lambda::new(
                vec![],
                types::None::new(Position::dummy()),
                None::new(Position::dummy()),
                Position::dummy(),
            ),
            false,
        );

        assert_eq!(
            compile_module(&Module::empty().set_definitions(vec![definition.clone(), definition])),
            Err(CompileError::DuplicateFunctionNames(
                Position::dummy(),
                Position::dummy()
            ))
        );
    }
}
