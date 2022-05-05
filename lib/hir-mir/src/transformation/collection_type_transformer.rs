use crate::{context::CompileContext, CompileError, ListTypeConfiguration};
use hir::types::{self, Type};
use position::Position;

pub fn transform_list(context: &CompileContext, position: &Position) -> Result<Type, CompileError> {
    Ok(transform_list_from_configuration(
        &context.configuration()?.list_type,
        position,
    ))
}

pub fn transform_list_from_configuration(
    configuration: &ListTypeConfiguration,
    position: &Position,
) -> Type {
    types::Reference::new(&configuration.list_type_name, position.clone()).into()
}

pub fn transform_map(context: &CompileContext, position: &Position) -> Result<Type, CompileError> {
    Ok(types::Reference::new(
        &context.configuration()?.map_type.map_type_name,
        position.clone(),
    )
    .into())
}

pub fn transform_map_context(
    context: &CompileContext,
    position: &Position,
) -> Result<Type, CompileError> {
    Ok(types::Reference::new(
        &context.configuration()?.map_type.context_type_name,
        position.clone(),
    )
    .into())
}
