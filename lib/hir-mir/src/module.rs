use super::{context::CompileContext, expression, generic_type_definition, type_, CompileError};
use crate::runtime_function_declaration;
use hir::{analysis::AnalysisError, ir::*};

pub fn compile(context: &CompileContext, module: &Module) -> Result<mir::ir::Module, CompileError> {
    Ok(mir::ir::Module::new(
        module
            .type_definitions()
            .iter()
            .map(|type_definition| compile_type_definition(type_definition, context))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .chain(generic_type_definition::compile(module, context)?)
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
                            AnalysisError::FunctionExpected(declaration.position().clone())
                        })?,
                    compile_calling_convention(declaration.calling_convention()),
                ))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .chain(if context.configuration().is_ok() {
                runtime_function_declaration::compile(context, module)?
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
            .map(|declaration| compile_function_declaration(declaration, context))
            .collect::<Result<_, _>>()?,
        module
            .function_definitions()
            .iter()
            .map(|definition| compile_function_definition(definition, context))
            .collect::<Result<Vec<_>, CompileError>>()?,
    ))
}

fn compile_calling_convention(calling_convention: CallingConvention) -> mir::ir::CallingConvention {
    match calling_convention {
        CallingConvention::Native => mir::ir::CallingConvention::Source,
        CallingConvention::C => mir::ir::CallingConvention::Target,
    }
}

fn compile_type_definition(
    type_definition: &TypeDefinition,
    context: &CompileContext,
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

fn compile_function_declaration(
    declaration: &FunctionDeclaration,
    context: &CompileContext,
) -> Result<mir::ir::FunctionDeclaration, CompileError> {
    Ok(mir::ir::FunctionDeclaration::new(
        declaration.name(),
        type_::compile_function(context, declaration.type_())?,
    ))
}

fn compile_function_definition(
    definition: &FunctionDefinition,
    context: &CompileContext,
) -> Result<mir::ir::FunctionDefinition, CompileError> {
    let body = expression::compile(context, definition.lambda().body())?;
    let result_type = type_::compile(context, definition.lambda().result_type())?;

    Ok(if definition.lambda().arguments().is_empty() {
        mir::ir::FunctionDefinition::thunk(definition.name(), body, result_type)
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
            body,
            result_type,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_configuration::COMPILE_CONFIGURATION;
    use hir::{test::ModuleFake, types};
    use mir::test::ModuleFake as _;
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    fn create_context(module: &Module) -> CompileContext {
        CompileContext::new(module, COMPILE_CONFIGURATION.clone().into())
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

        assert_eq!(
            compile(&context, &module),
            Ok(mir::ir::Module::empty()
                .set_foreign_declarations(
                    runtime_function_declaration::compile(&context, &module).unwrap()
                )
                .set_foreign_definitions(vec![mir::ir::ForeignDefinition::new(
                    "foo",
                    "bar",
                    mir::ir::CallingConvention::Source
                )],)
                .set_function_definitions(vec![mir::ir::FunctionDefinition::new(
                    "foo",
                    vec![mir::ir::Argument::new("x", mir::types::Type::None)],
                    mir::ir::Expression::None,
                    mir::types::Type::None,
                )],))
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

        assert_eq!(
            compile(&context, &module),
            Ok(mir::ir::Module::empty()
                .set_foreign_declarations(
                    runtime_function_declaration::compile(&context, &module).unwrap()
                )
                .set_foreign_definitions(vec![mir::ir::ForeignDefinition::new(
                    "foo",
                    "bar",
                    mir::ir::CallingConvention::Target
                )])
                .set_function_definitions(vec![mir::ir::FunctionDefinition::new(
                    "foo",
                    vec![mir::ir::Argument::new("x", mir::types::Type::None)],
                    mir::ir::Expression::None,
                    mir::types::Type::None,
                )],))
        );
    }
}
