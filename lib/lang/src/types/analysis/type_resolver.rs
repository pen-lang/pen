use super::{super::*, TypeAnalysisError};
use std::collections::HashMap;

pub fn resolve_type(
    type_: &Type,
    types: &HashMap<String, Type>,
) -> Result<Type, TypeAnalysisError> {
    Ok(match type_ {
        Type::Reference(reference) => resolve_reference(reference, types)?,
        _ => type_.clone(),
    })
}

pub fn resolve_reference(
    reference: &Reference,
    types: &HashMap<String, Type>,
) -> Result<Type, TypeAnalysisError> {
    Ok(
        match types
            .get(reference.name())
            .ok_or_else(|| TypeAnalysisError::TypeNotFound(reference.clone()))?
        {
            Type::Reference(reference) => resolve_reference(reference, types)?,
            type_ => type_.clone(),
        },
    )
}

pub fn resolve_to_function(
    type_: &Type,
    types: &HashMap<String, Type>,
) -> Result<Option<Function>, TypeAnalysisError> {
    Ok(resolve_type(type_, types)?.into_function())
}

pub fn resolve_to_list(
    type_: &Type,
    types: &HashMap<String, Type>,
) -> Result<Option<List>, TypeAnalysisError> {
    Ok(resolve_type(type_, types)?.into_list())
}

pub fn resolve_to_record(
    type_: &Type,
    types: &HashMap<String, Type>,
) -> Result<Option<Record>, TypeAnalysisError> {
    Ok(resolve_type(type_, types)?.into_record())
}

pub fn resolve_record_elements<'a>(
    type_: &Type,
    types: &HashMap<String, Type>,
    records: &'a HashMap<String, HashMap<String, Type>>,
) -> Result<Option<&'a HashMap<String, Type>>, TypeAnalysisError> {
    Ok(if let Some(record) = resolve_to_record(type_, types)? {
        Some(
            records
                .get(record.name())
                .ok_or_else(|| TypeAnalysisError::RecordNotFound(record.clone()))?,
        )
    } else {
        None
    })
}
