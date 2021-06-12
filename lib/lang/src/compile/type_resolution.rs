use super::{type_context::TypeContext, CompileError};
use crate::types::{self, Type};

pub fn resolve_to_function(
    type_: &Type,
    context: &TypeContext,
) -> Result<Option<types::Function>, CompileError> {
    Ok(resolve_type(type_, context)?.into_function())
}

pub fn resolve_reference(
    reference: &types::Reference,
    context: &TypeContext,
) -> Result<Type, CompileError> {
    Ok(
        match context
            .types()
            .get(reference.name())
            .ok_or_else(|| CompileError::TypeNotFound(reference.clone()))?
        {
            Type::Reference(reference) => resolve_reference(reference, context)?,
            type_ => type_.clone(),
        },
    )
}

fn resolve_type(type_: &Type, context: &TypeContext) -> Result<Type, CompileError> {
    Ok(match type_ {
        Type::Reference(reference) => resolve_reference(reference, context)?,
        _ => type_.clone(),
    })
}
