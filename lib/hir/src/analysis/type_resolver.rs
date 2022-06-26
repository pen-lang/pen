use super::AnalysisError;
use crate::types::*;
use fnv::FnvHashMap;

pub fn resolve(
    reference: &Reference,
    types: &FnvHashMap<String, Type>,
) -> Result<Type, AnalysisError> {
    Ok(types
        .get(reference.name())
        .ok_or_else(|| AnalysisError::TypeNotFound(reference.clone()))?
        .clone()
        .set_position(reference.position().clone()))
}
