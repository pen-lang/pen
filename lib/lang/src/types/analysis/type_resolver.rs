use super::{super::*, TypeError};
use std::collections::HashMap;

pub fn resolve(type_: &Type, types: &HashMap<String, Type>) -> Result<Type, TypeError> {
    Ok(match type_ {
        Type::Reference(reference) => resolve(
            types
                .get(reference.name())
                .ok_or_else(|| TypeError::TypeNotFound(reference.clone()))?,
            types,
        )?,
        _ => type_.clone(),
    })
}

pub fn resolve_function(
    type_: &Type,
    types: &HashMap<String, Type>,
) -> Result<Option<Function>, TypeError> {
    Ok(resolve(type_, types)?.into_function())
}

pub fn resolve_list(
    type_: &Type,
    types: &HashMap<String, Type>,
) -> Result<Option<List>, TypeError> {
    Ok(resolve(type_, types)?.into_list())
}

pub fn resolve_record(
    type_: &Type,
    types: &HashMap<String, Type>,
) -> Result<Option<Record>, TypeError> {
    Ok(resolve(type_, types)?.into_record())
}
