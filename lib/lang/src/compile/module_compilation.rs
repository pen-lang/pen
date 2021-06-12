use super::{type_compilation, type_context::TypeContext, CompileError};
use crate::{compile::expressions, hir::*};
use std::collections::HashMap;

pub fn compile(
    module: &Module,
    type_context: &TypeContext,
) -> Result<mir::ir::Module, CompileError> {
    let variables = module
        .definitions()
        .iter()
        .map(|definition| {
            Ok((
                definition.name().into(),
                mir::ir::Call::new(
                    mir::types::Function::new(
                        vec![],
                        type_compilation::compile(definition.type_(), type_context)?,
                    ),
                    mir::ir::Variable::new(definition.name()),
                    vec![],
                )
                .into(),
            ))
        })
        .collect::<Result<_, CompileError>>()?;

    Ok(mir::ir::Module::new(
        module
            .type_definitions()
            .iter()
            .map(|type_definition| compile_type_definition(type_definition, type_context))
            .collect::<Result<_, _>>()?,
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
            .map(|definition| compile_definition(definition, &variables, type_context))
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
        mir::types::Function::new(
            vec![],
            type_compilation::compile(declaration.type_(), type_context)?,
        ),
    ))
}

fn compile_definition(
    definition: &Definition,
    variables: &HashMap<String, mir::ir::Expression>,
    type_context: &TypeContext,
) -> Result<mir::ir::Definition, CompileError> {
    Ok(mir::ir::Definition::new(
        definition.name(),
        vec![],
        expressions::compile(definition.body(), variables, type_context)?,
        type_compilation::compile(definition.type_(), type_context)?,
    ))
}
