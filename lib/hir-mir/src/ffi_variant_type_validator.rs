use crate::{context::CompileContext, error::CompileError};
use hir::{
    analysis::{type_canonicalizer, AnalysisError},
    ir::*,
    types::Type,
};

// Validate variant types in FFI because the current backend cannot handle them
// properly for some targets (e.g. i386.)
pub fn validate(context: &CompileContext, module: &Module) -> Result<(), CompileError> {
    for declaration in module.foreign_declarations() {
        validate_foreign_declaration(context, declaration)?;
    }

    for definition in module.function_definitions() {
        validate_definition(context, definition)?;
    }

    Ok(())
}

fn validate_foreign_declaration(
    context: &CompileContext,
    declaration: &ForeignDeclaration,
) -> Result<(), CompileError> {
    let function_type =
        type_canonicalizer::canonicalize_function(declaration.type_(), context.types())?
            .ok_or_else(|| {
                AnalysisError::FunctionExpected(declaration.type_().position().clone())
            })?;

    for argument_type in function_type.arguments() {
        validate_type(context, argument_type)?;
    }

    validate_type(context, function_type.result())?;

    Ok(())
}

fn validate_definition(
    context: &CompileContext,
    definition: &FunctionDefinition,
) -> Result<(), CompileError> {
    if definition.foreign_definition_configuration().is_none() {
        return Ok(());
    }

    for argument in definition.lambda().arguments() {
        validate_type(context, argument.type_())?;
    }

    validate_type(context, definition.lambda().result_type())?;

    Ok(())
}

fn validate_type(context: &CompileContext, type_: &Type) -> Result<(), CompileError> {
    if type_canonicalizer::canonicalize(type_, context.types())?.is_variant() {
        return Err(CompileError::VariantTypeInFfi(type_.position().clone()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_configuration::COMPILE_CONFIGURATION;
    use hir::{test::ModuleFake, types};
    use position::{test::PositionFake, Position};

    fn validate_module(module: &Module) -> Result<(), CompileError> {
        validate(
            &CompileContext::new(module, COMPILE_CONFIGURATION.clone().into()),
            module,
        )
    }

    #[test]
    fn validate_empty_module() -> Result<(), CompileError> {
        validate_module(&Module::empty())
    }

    #[test]
    fn fail_to_validate_foreign_declaration_argument() {
        assert_eq!(
            validate_module(&Module::empty().set_foreign_declarations(vec![
                ForeignDeclaration::new(
                    "f",
                    "f",
                    CallingConvention::C,
                    types::Function::new(
                        vec![types::Any::new(Position::fake()).into()],
                        types::Number::new(Position::fake()),
                        Position::fake()
                    ),
                    Position::fake(),
                )
            ])),
            Err(CompileError::VariantTypeInFfi(Position::fake()))
        );
    }

    #[test]
    fn fail_to_validate_foreign_declaration_return_value() {
        assert_eq!(
            validate_module(&Module::empty().set_foreign_declarations(vec![
                ForeignDeclaration::new(
                    "f",
                    "f",
                    CallingConvention::C,
                    types::Function::new(
                        vec![],
                        types::Any::new(Position::fake()),
                        Position::fake()
                    ),
                    Position::fake(),
                )
            ])),
            Err(CompileError::VariantTypeInFfi(Position::fake()))
        );
    }

    #[test]
    fn fail_to_validate_definition_argument() {
        assert_eq!(
            validate_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::new(
                    "f",
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::Any::new(Position::fake()))],
                        types::None::new(Position::fake()),
                        None::new(Position::fake()),
                        Position::fake(),
                    ),
                    Some(ForeignDefinitionConfiguration::new(CallingConvention::C)),
                    false,
                    Position::fake(),
                )])
            ),
            Err(CompileError::VariantTypeInFfi(Position::fake()))
        );
    }

    #[test]
    fn fail_to_validate_definition_return_value() {
        assert_eq!(
            validate_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::new(
                    "f",
                    "f",
                    Lambda::new(
                        vec![],
                        types::Any::new(Position::fake()),
                        None::new(Position::fake()),
                        Position::fake(),
                    ),
                    Some(ForeignDefinitionConfiguration::new(CallingConvention::C)),
                    false,
                    Position::fake(),
                )])
            ),
            Err(CompileError::VariantTypeInFfi(Position::fake()))
        );
    }

    #[test]
    fn validate_non_foreign_definition() {
        assert_eq!(
            validate_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::new(
                    "f",
                    "f",
                    Lambda::new(
                        vec![],
                        types::Any::new(Position::fake()),
                        None::new(Position::fake()),
                        Position::fake(),
                    ),
                    None,
                    false,
                    Position::fake(),
                )])
            ),
            Ok(())
        );
    }
}
