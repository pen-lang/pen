use super::{
    compile_context::CompileContext, expression_compiler, generic_type_definition_compiler,
    type_compiler, CompileError,
};
use crate::spawn_function_declaration_compiler;
use hir::ir::*;

pub fn compile(
    module: &Module,
    compile_context: &CompileContext,
) -> Result<mir::ir::Module, CompileError> {
    Ok(mir::ir::Module::new(
        module
            .type_definitions()
            .iter()
            .map(|type_definition| compile_type_definition(type_definition, compile_context))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .chain(generic_type_definition_compiler::compile(
                module,
                compile_context,
            )?)
            .collect(),
        module
            .foreign_declarations()
            .iter()
            .map(|declaration| -> Result<_, CompileError> {
                Ok(mir::ir::ForeignDeclaration::new(
                    declaration.name(),
                    declaration.foreign_name(),
                    type_compiler::compile(declaration.type_(), compile_context)?
                        .into_function()
                        .ok_or_else(|| {
                            CompileError::FunctionExpected(declaration.position().clone())
                        })?,
                    compile_calling_convention(declaration.calling_convention()),
                ))
            })
            .chain(compile_context.configuration().ok().map(|configuration| {
                Ok(spawn_function_declaration_compiler::compile(
                    &configuration.concurrency,
                ))
            }))
            .collect::<Result<_, _>>()?,
        module
            .definitions()
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
            .declarations()
            .iter()
            .map(|declaration| compile_declaration(declaration, compile_context))
            .collect::<Result<_, _>>()?,
        module
            .definitions()
            .iter()
            .map(|definition| compile_definition(definition, compile_context))
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
    compile_context: &CompileContext,
) -> Result<mir::ir::TypeDefinition, CompileError> {
    Ok(mir::ir::TypeDefinition::new(
        type_definition.name(),
        mir::types::RecordBody::new(
            type_definition
                .fields()
                .iter()
                .map(|field| type_compiler::compile(field.type_(), compile_context))
                .collect::<Result<_, _>>()?,
        ),
    ))
}

fn compile_declaration(
    declaration: &Declaration,
    compile_context: &CompileContext,
) -> Result<mir::ir::Declaration, CompileError> {
    Ok(mir::ir::Declaration::new(
        declaration.name(),
        type_compiler::compile_function(declaration.type_(), compile_context)?,
    ))
}

fn compile_definition(
    definition: &Definition,
    compile_context: &CompileContext,
) -> Result<mir::ir::Definition, CompileError> {
    let body = expression_compiler::compile(definition.lambda().body(), compile_context)?;
    let result_type = type_compiler::compile(definition.lambda().result_type(), compile_context)?;

    Ok(mir::ir::Definition::new(
        definition.name(),
        definition
            .lambda()
            .arguments()
            .iter()
            .map(|argument| -> Result<_, CompileError> {
                Ok(mir::ir::Argument::new(
                    argument.name(),
                    type_compiler::compile(argument.type_(), compile_context)?,
                ))
            })
            .collect::<Result<_, _>>()?,
        body,
        result_type,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        compile_configuration::COMPILE_CONFIGURATION,
        concurrency_configuration::CONCURRENCY_CONFIGURATION,
    };
    use hir::{test::ModuleFake, types};
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    fn compile_module(module: &Module) -> Result<mir::ir::Module, CompileError> {
        compile(
            module,
            &CompileContext::new(module, COMPILE_CONFIGURATION.clone().into()),
        )
    }

    #[test]
    fn compile_foreign_definition() {
        assert_eq!(
            compile_module(&Module::empty().set_definitions(vec![Definition::new(
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
            )])),
            Ok(mir::ir::Module::new(
                vec![],
                vec![spawn_function_declaration_compiler::compile(
                    &CONCURRENCY_CONFIGURATION
                )],
                vec![mir::ir::ForeignDefinition::new(
                    "foo",
                    "bar",
                    mir::ir::CallingConvention::Source
                )],
                vec![],
                vec![mir::ir::Definition::new(
                    "foo",
                    vec![mir::ir::Argument::new("x", mir::types::Type::None)],
                    mir::ir::Expression::None,
                    mir::types::Type::None,
                )],
            ))
        );
    }

    #[test]
    fn compile_foreign_definition_with_c_calling_convention() {
        assert_eq!(
            compile_module(&Module::empty().set_definitions(vec![Definition::new(
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
            )])),
            Ok(mir::ir::Module::new(
                vec![],
                vec![spawn_function_declaration_compiler::compile(
                    &CONCURRENCY_CONFIGURATION
                )],
                vec![mir::ir::ForeignDefinition::new(
                    "foo",
                    "bar",
                    mir::ir::CallingConvention::Target
                )],
                vec![],
                vec![mir::ir::Definition::new(
                    "foo",
                    vec![mir::ir::Argument::new("x", mir::types::Type::None)],
                    mir::ir::Expression::None,
                    mir::types::Type::None,
                )],
            ))
        );
    }
}
