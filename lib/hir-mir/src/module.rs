use super::{CompileError, context::Context, expression, generic_type_definition, type_};
use crate::{error_type, runtime_function_declaration, type_information};
use hir::{analysis::AnalysisError, ir::*};

pub fn compile(context: &Context, module: &Module) -> Result<mir::ir::Module, CompileError> {
    let (type_information_function_declarations, type_information_function_definitions) =
        type_information::compile_function_declarations_and_definitions(context, module)?;

    Ok(mir::ir::Module::new(
        module
            .type_definitions()
            .iter()
            .map(|type_definition| compile_type_definition(context, type_definition))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .chain(generic_type_definition::compile(context, module)?)
            .chain(compile_internal_type_definitions())
            .collect(),
        module
            .foreign_declarations()
            .iter()
            .map(|declaration| -> Result<_, CompileError> {
                Ok(mir::ir::ForeignDeclaration::new(
                    declaration.name(),
                    declaration.foreign_name(),
                    type_::compile(context, declaration.type_())?
                        .into_function()
                        .ok_or_else(|| {
                            AnalysisError::FunctionExpected(
                                declaration.position().clone(),
                                declaration.type_().clone(),
                            )
                        })?,
                    compile_calling_convention(declaration.calling_convention()),
                ))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .chain(if context.configuration().is_ok() {
                runtime_function_declaration::compile(context)?
            } else {
                vec![]
            })
            .collect(),
        module
            .function_definitions()
            .iter()
            .flat_map(|definition| {
                definition
                    .foreign_definition_configuration()
                    .map(|configuration| {
                        mir::ir::ForeignDefinition::new(
                            definition.name(),
                            definition.original_name(),
                            compile_calling_convention(configuration.calling_convention()),
                        )
                    })
            })
            .collect(),
        module
            .function_declarations()
            .iter()
            .map(|declaration| compile_function_declaration(context, declaration))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .chain(type_information_function_declarations)
            .collect(),
        module
            .function_definitions()
            .iter()
            .map(|definition| compile_function_definition(context, definition))
            .collect::<Result<Vec<_>, CompileError>>()?
            .into_iter()
            .chain(type_information_function_definitions)
            .collect(),
        type_information::compile_type_information(context, module)?,
    ))
}

fn compile_calling_convention(calling_convention: CallingConvention) -> mir::ir::CallingConvention {
    match calling_convention {
        CallingConvention::Native => mir::ir::CallingConvention::Source,
        CallingConvention::C => mir::ir::CallingConvention::Target,
    }
}

fn compile_type_definition(
    context: &Context,
    type_definition: &TypeDefinition,
) -> Result<mir::ir::TypeDefinition, CompileError> {
    Ok(mir::ir::TypeDefinition::new(
        type_definition.name(),
        mir::types::RecordBody::new(
            type_definition
                .fields()
                .iter()
                .map(|field| type_::compile(context, field.type_()))
                .collect::<Result<_, _>>()?,
        ),
    ))
}

fn compile_internal_type_definitions() -> Vec<mir::ir::TypeDefinition> {
    vec![
        error_type::compile_type_definition(),
        type_information::compile_type_information_type_definition(),
    ]
}

fn compile_function_declaration(
    context: &Context,
    declaration: &FunctionDeclaration,
) -> Result<mir::ir::FunctionDeclaration, CompileError> {
    Ok(mir::ir::FunctionDeclaration::new(
        declaration.name(),
        type_::compile_function(context, declaration.type_())?,
    ))
}

fn compile_function_definition(
    context: &Context,
    definition: &FunctionDefinition,
) -> Result<mir::ir::GlobalFunctionDefinition, CompileError> {
    let body = expression::compile(context, definition.lambda().body())?;
    let result_type = type_::compile(context, definition.lambda().result_type())?;

    Ok(mir::ir::GlobalFunctionDefinition::new(
        if definition.lambda().arguments().is_empty() {
            mir::ir::FunctionDefinition::thunk(definition.name(), result_type, body)
        } else {
            mir::ir::FunctionDefinition::new(
                definition.name(),
                definition
                    .lambda()
                    .arguments()
                    .iter()
                    .map(|argument| -> Result<_, CompileError> {
                        Ok(mir::ir::Argument::new(
                            argument.name(),
                            type_::compile(context, argument.type_())?,
                        ))
                    })
                    .collect::<Result<_, _>>()?,
                result_type,
                body,
            )
        },
        definition.is_public(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_configuration::COMPILE_CONFIGURATION;
    use hir::{test::ModuleFake, types};
    use mir::test::{GlobalFunctionDefinitionFake, ModuleFake as _};
    use position::{Position, test::PositionFake};
    use pretty_assertions::assert_eq;

    fn create_context(module: &Module) -> Context {
        Context::new(module, COMPILE_CONFIGURATION.clone().into())
    }

    #[test]
    fn compile_foreign_definition() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::new(
            "foo",
            "bar",
            Lambda::new(
                vec![Argument::new("x", types::None::new(Position::fake()))],
                types::None::new(Position::fake()),
                None::new(Position::fake()),
                Position::fake(),
            ),
            ForeignDefinitionConfiguration::new(CallingConvention::Native).into(),
            false,
            Position::fake(),
        )]);
        let context = create_context(&module);
        let (type_information_function_declarations, type_information_function_definitions) =
            type_information::compile_function_declarations_and_definitions(&context, &module)
                .unwrap();

        assert_eq!(
            compile(&context, &module),
            Ok(mir::ir::Module::empty()
                .set_type_information(
                    type_information::compile_type_information(&context, &module).unwrap()
                )
                .set_foreign_declarations(runtime_function_declaration::compile(&context).unwrap())
                .set_foreign_definitions(vec![mir::ir::ForeignDefinition::new(
                    "foo",
                    "bar",
                    mir::ir::CallingConvention::Source
                )])
                .set_type_definitions(compile_internal_type_definitions())
                .set_function_declarations(type_information_function_declarations)
                .set_global_function_definitions(
                    [mir::ir::GlobalFunctionDefinition::fake(
                        mir::ir::FunctionDefinition::new(
                            "foo",
                            vec![mir::ir::Argument::new("x", mir::types::Type::None)],
                            mir::types::Type::None,
                            mir::ir::Expression::None,
                        )
                    )]
                    .into_iter()
                    .chain(type_information_function_definitions)
                    .collect()
                ))
        );
    }

    #[test]
    fn compile_foreign_definition_with_c_calling_convention() {
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::new(
            "foo",
            "bar",
            Lambda::new(
                vec![Argument::new("x", types::None::new(Position::fake()))],
                types::None::new(Position::fake()),
                None::new(Position::fake()),
                Position::fake(),
            ),
            ForeignDefinitionConfiguration::new(CallingConvention::C).into(),
            false,
            Position::fake(),
        )]);
        let context = create_context(&module);
        let (type_information_function_declarations, type_information_function_definitions) =
            type_information::compile_function_declarations_and_definitions(&context, &module)
                .unwrap();

        assert_eq!(
            compile(&context, &module),
            Ok(mir::ir::Module::empty()
                .set_type_information(
                    type_information::compile_type_information(&context, &module).unwrap()
                )
                .set_foreign_declarations(runtime_function_declaration::compile(&context).unwrap())
                .set_foreign_definitions(vec![mir::ir::ForeignDefinition::new(
                    "foo",
                    "bar",
                    mir::ir::CallingConvention::Target
                )])
                .set_type_definitions(compile_internal_type_definitions())
                .set_function_declarations(type_information_function_declarations)
                .set_global_function_definitions(
                    [mir::ir::GlobalFunctionDefinition::fake(
                        mir::ir::FunctionDefinition::new(
                            "foo",
                            vec![mir::ir::Argument::new("x", mir::types::Type::None)],
                            mir::types::Type::None,
                            mir::ir::Expression::None,
                        )
                    )]
                    .into_iter()
                    .chain(type_information_function_definitions)
                    .collect()
                ))
        );
    }
}
