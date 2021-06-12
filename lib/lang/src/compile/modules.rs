use super::list_type_configuration::ListTypeConfiguration;
use super::type_compilation;
use super::type_context::TypeContext;
use super::CompileError;
use crate::compile::expressions;
use crate::hir::*;
use crate::types::Type;
use std::collections::HashMap;

pub fn compile(
    module: &Module,
    type_context: &TypeContext,
) -> Result<mir::ir::Module, CompileError> {
    Ok(mir::ir::Module::new(
        module
            .type_definitions()
            .iter()
            .map(|type_definition| compile_type_definition(type_definition, type_context))
            .collect::<Result<_, _>>()?,
        todo!(),
        todo!(),
        todo!(),
        module
            .definitions()
            .iter()
            .map(|definition| compile_definition(definition, type_context))
            .collect::<Result<Vec<_>, CompileError>>()?,
    ))
}

fn compile_definition(
    definition: &Definition,
    type_context: &TypeContext,
) -> Result<mir::ir::Definition, CompileError> {
    Ok(mir::ir::Definition::new(
        definition.name(),
        vec![],
        expressions::compile(definition.body(), type_context)?,
        type_compilation::compile(definition.type_(), type_context)?,
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
