mod environment_creator;
mod error;
mod expression_compiler;
mod list_type_configuration;
mod main_function_compiler;
mod main_module_configuration;
mod module_compiler;
mod module_interface_compiler;
mod string_type_configuration;
mod transformation;
mod type_checker;
mod type_coercer;
mod type_compiler;
mod type_context;
mod type_extractor;
mod type_inferrer;

use self::{transformation::record_equal_function_transformer, type_context::TypeContext};
use crate::{hir::*, interface};
pub use error::CompileError;
pub use list_type_configuration::ListTypeConfiguration;
pub use main_module_configuration::MainModuleConfiguration;
pub use string_type_configuration::StringTypeConfiguration;

pub fn compile_main(
    module: &Module,
    list_type_configuration: &ListTypeConfiguration,
    string_type_configuration: &StringTypeConfiguration,
    main_module_configuration: &MainModuleConfiguration,
) -> Result<mir::ir::Module, CompileError> {
    let type_context = TypeContext::new(module, list_type_configuration, string_type_configuration);
    let module =
        main_function_compiler::compile(module, type_context.types(), main_module_configuration)?;
    let (module, _) = compile_module(&module, &type_context)?;

    Ok(module)
}

pub fn compile(
    module: &Module,
    list_type_configuration: &ListTypeConfiguration,
    string_type_configuration: &StringTypeConfiguration,
) -> Result<(mir::ir::Module, interface::Module), CompileError> {
    compile_module(
        module,
        &TypeContext::new(module, list_type_configuration, string_type_configuration),
    )
}

fn compile_module(
    module: &Module,
    type_context: &TypeContext,
) -> Result<(mir::ir::Module, interface::Module), CompileError> {
    let module = record_equal_function_transformer::transform(module, type_context)?;
    let module = type_inferrer::infer_types(&module, type_context)?;
    type_checker::check_types(&module, type_context)?;
    let module = type_coercer::coerce_types(&module, type_context)?;

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
    use super::{list_type_configuration::LIST_TYPE_CONFIGURATION, *};
    use crate::{
        hir_mir::string_type_configuration::STRING_TYPE_CONFIGURATION, position::Position, types,
    };

    fn compile_module(
        module: &Module,
    ) -> Result<(mir::ir::Module, interface::Module), CompileError> {
        compile(module, &LIST_TYPE_CONFIGURATION, &STRING_TYPE_CONFIGURATION)
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
    fn compile_record() -> Result<(), CompileError> {
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
        )?;

        Ok(())
    }
}
