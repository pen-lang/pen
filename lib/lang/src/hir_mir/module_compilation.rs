use super::{expression_compilation, type_compilation::NONE_RECORD_TYPE_NAME};
use super::{type_compilation, type_context::TypeContext, CompileError};
use crate::hir::*;

pub fn compile(
    module: &Module,
    type_context: &TypeContext,
) -> Result<mir::ir::Module, CompileError> {
    Ok(mir::ir::Module::new(
        vec![mir::ir::TypeDefinition::new(
            NONE_RECORD_TYPE_NAME,
            mir::types::RecordBody::new(vec![]),
        )]
        .into_iter()
        .chain(
            module
                .type_definitions()
                .iter()
                .map(|type_definition| compile_type_definition(type_definition, type_context))
                .collect::<Result<Vec<_>, _>>()?,
        )
        .collect(),
        vec![],
        vec![],
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
                .map(|element| type_compilation::compile(element.type_(), type_context))
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
        type_compilation::compile_function(declaration.type_(), type_context)?,
    ))
}

fn compile_definition(
    definition: &Definition,
    type_context: &TypeContext,
) -> Result<mir::ir::Definition, CompileError> {
    let body = expression_compilation::compile_block(definition.lambda().body(), type_context)?;
    let result_type = type_compilation::compile(definition.lambda().result_type(), type_context)?;

    Ok(if definition.lambda().arguments().is_empty() {
        mir::ir::Definition::thunk(definition.name(), vec![], body, result_type)
    } else {
        mir::ir::Definition::new(
            definition.name(),
            definition
                .lambda()
                .arguments()
                .iter()
                .map(|argument| -> Result<_, CompileError> {
                    Ok(mir::ir::Argument::new(
                        argument.name(),
                        type_compilation::compile(argument.type_(), type_context)?,
                    ))
                })
                .collect::<Result<_, _>>()?,
            body,
            result_type,
        )
    })
}
