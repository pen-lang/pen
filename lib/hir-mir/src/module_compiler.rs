use super::{
    expression_compiler, generic_type_definition_compiler, type_compiler,
    type_context::TypeContext, CompileError,
};
use hir::ir::*;

pub fn compile(
    module: &Module,
    type_context: &TypeContext,
) -> Result<mir::ir::Module, CompileError> {
    Ok(mir::ir::Module::new(
        module
            .type_definitions()
            .iter()
            .map(|type_definition| compile_type_definition(type_definition, type_context))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .chain(generic_type_definition_compiler::compile(
                module,
                type_context,
            )?)
            .collect(),
        module
            .foreign_declarations()
            .iter()
            .map(|declaration| -> Result<_, CompileError> {
                Ok(mir::ir::ForeignDeclaration::new(
                    declaration.name(),
                    declaration.foreign_name(),
                    type_compiler::compile(declaration.type_(), type_context)?
                        .into_function()
                        .ok_or_else(|| {
                            CompileError::FunctionExpected(declaration.position().clone())
                        })?,
                    match declaration.calling_convention() {
                        CallingConvention::Native => mir::ir::CallingConvention::Source,
                        CallingConvention::C => mir::ir::CallingConvention::Target,
                    },
                ))
            })
            .collect::<Result<_, _>>()?,
        module
            .definitions()
            .iter()
            .flat_map(|definition| {
                if definition.is_foreign() {
                    Some(mir::ir::ForeignDefinition::new(
                        definition.name(),
                        definition.original_name(),
                    ))
                } else {
                    None
                }
            })
            .collect(),
        module
            .declarations()
            .iter()
            .map(|declaration| compile_declaration(declaration, type_context))
            .collect::<Result<_, _>>()?,
        module
            .definitions()
            .iter()
            .map(|definition| compile_definition(definition, type_context))
            .collect::<Result<Vec<_>, CompileError>>()?,
    ))
}

fn compile_type_definition(
    type_definition: &TypeDefinition,
    type_context: &TypeContext,
) -> Result<mir::ir::TypeDefinition, CompileError> {
    Ok(mir::ir::TypeDefinition::new(
        type_definition.name(),
        mir::types::RecordBody::new(
            type_definition
                .elements()
                .iter()
                .map(|element| type_compiler::compile(element.type_(), type_context))
                .collect::<Result<_, _>>()?,
        ),
    ))
}

fn compile_declaration(
    declaration: &Declaration,
    type_context: &TypeContext,
) -> Result<mir::ir::Declaration, CompileError> {
    Ok(mir::ir::Declaration::new(
        declaration.name(),
        type_compiler::compile_function(declaration.type_(), type_context)?,
    ))
}

fn compile_definition(
    definition: &Definition,
    type_context: &TypeContext,
) -> Result<mir::ir::Definition, CompileError> {
    let body = expression_compiler::compile(definition.lambda().body(), type_context)?;
    let result_type = type_compiler::compile(definition.lambda().result_type(), type_context)?;

    Ok(mir::ir::Definition::new(
        definition.name(),
        definition
            .lambda()
            .arguments()
            .iter()
            .map(|argument| -> Result<_, CompileError> {
                Ok(mir::ir::Argument::new(
                    argument.name(),
                    type_compiler::compile(argument.type_(), type_context)?,
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
        error_type_configuration::ERROR_TYPE_CONFIGURATION,
        list_type_configuration::LIST_TYPE_CONFIGURATION,
        string_type_configuration::STRING_TYPE_CONFIGURATION,
    };
    use hir::{test::ModuleFake, types};
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    fn compile_module(module: &Module) -> Result<mir::ir::Module, CompileError> {
        compile(
            module,
            &TypeContext::new(
                module,
                &LIST_TYPE_CONFIGURATION,
                &STRING_TYPE_CONFIGURATION,
                &ERROR_TYPE_CONFIGURATION,
            ),
        )
    }

    #[test]
    fn compile_foreign_declaration() {
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
                true,
                false,
                Position::fake(),
            )])),
            Ok(mir::ir::Module::new(
                vec![],
                vec![],
                vec![mir::ir::ForeignDefinition::new("foo", "bar")],
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
