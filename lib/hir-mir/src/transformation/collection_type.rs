use crate::{context::CompileContext, CompileError};
use hir::types::{self, Type};
use position::Position;

pub fn transform_list(context: &CompileContext, position: &Position) -> Result<Type, CompileError> {
    Ok(types::Reference::new(
        &context.configuration()?.list_type.list_type_name,
        position.clone(),
    )
    .into())
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
