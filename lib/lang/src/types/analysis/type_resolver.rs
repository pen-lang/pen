use super::{super::*, TypeError};
use crate::{position::Position, types};
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

pub fn resolve_record_elements<'a>(
    type_: &Type,
    position: &Position,
    types: &HashMap<String, Type>,
    records: &'a HashMap<String, Vec<types::RecordElement>>,
) -> Result<&'a [types::RecordElement], TypeError> {
    let record =
        resolve_record(type_, types)?.ok_or_else(|| TypeError::RecordExpected(position.clone()))?;

    Ok(records
        .get(record.name())
        .ok_or_else(|| TypeError::RecordNotFound(record.clone()))?)
}
