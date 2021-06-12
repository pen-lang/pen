use super::CompileError;
use crate::types::{self, Type};
use std::collections::HashMap;

pub fn resolve_type(type_: &Type, types: &HashMap<String, Type>) -> Result<Type, CompileError> {
    Ok(match type_ {
        Type::Reference(reference) => resolve_reference(reference, types)?,
        _ => type_.clone(),
    })
}

pub fn resolve_reference(
    reference: &types::Reference,
    types: &HashMap<String, Type>,
) -> Result<Type, CompileError> {
    Ok(
        match types
            .get(reference.name())
            .ok_or_else(|| CompileError::TypeNotFound(reference.clone()))?
        {
            Type::Reference(reference) => resolve_reference(reference, types)?,
            type_ => type_.clone(),
        },
    )
}

pub fn resolve_to_function(
    type_: &Type,
    types: &HashMap<String, Type>,
) -> Result<Option<types::Function>, CompileError> {
    Ok(resolve_type(type_, types)?.into_function())
}
