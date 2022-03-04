use super::AnalysisError;
use crate::types::*;
use fnv::FnvHashMap;

pub fn resolve(
    reference: &Reference,
    types: &FnvHashMap<String, Type>,
) -> Result<Type, AnalysisError> {
    Ok(resolve_type(&reference.clone().into(), types)?.set_position(reference.position().clone()))
}

fn resolve_type(type_: &Type, types: &FnvHashMap<String, Type>) -> Result<Type, AnalysisError> {
    Ok(match type_ {
        Type::Reference(reference) => resolve_type(
            types
                .get(reference.name())
                .ok_or_else(|| AnalysisError::TypeNotFound(reference.clone()))?,
            types,
        )?,
        _ => type_.clone(),
    })
}
