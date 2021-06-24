use super::{super::*, TypeError};
use crate::position::Position;
use std::collections::HashMap;

pub fn resolve_type(type_: &Type, types: &HashMap<String, Type>) -> Result<Type, TypeError> {
    Ok(match type_ {
        Type::Reference(reference) => resolve_reference(reference, types)?,
        _ => type_.clone(),
    })
}

pub fn resolve_reference(
    reference: &Reference,
    types: &HashMap<String, Type>,
) -> Result<Type, TypeError> {
    Ok(
        match types
            .get(reference.name())
            .ok_or_else(|| TypeError::TypeNotFound(reference.clone()))?
        {
            Type::Reference(reference) => resolve_reference(reference, types)?,
            type_ => type_.clone(),
        },
    )
}

pub fn resolve_to_function(
    type_: &Type,
    types: &HashMap<String, Type>,
) -> Result<Option<Function>, TypeError> {
    Ok(resolve_type(type_, types)?.into_function())
}

pub fn resolve_to_list(
    type_: &Type,
    types: &HashMap<String, Type>,
) -> Result<Option<List>, TypeError> {
    Ok(resolve_type(type_, types)?.into_list())
}

pub fn resolve_to_record(
    type_: &Type,
    types: &HashMap<String, Type>,
) -> Result<Option<Record>, TypeError> {
    Ok(resolve_type(type_, types)?.into_record())
}

pub fn resolve_record_elements<'a>(
    type_: &Type,
    position: &Position,
    types: &HashMap<String, Type>,
    records: &'a HashMap<String, HashMap<String, Type>>,
) -> Result<&'a HashMap<String, Type>, TypeError> {
    let record = resolve_to_record(type_, types)?
        .ok_or_else(|| TypeError::RecordExpected(position.clone()))?;

    records
        .get(record.name())
        .ok_or_else(|| TypeError::RecordNotFound(record.clone()))
}
