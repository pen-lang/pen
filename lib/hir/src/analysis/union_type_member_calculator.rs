use super::{type_canonicalizer, AnalysisError};
use crate::types::Type;
use fnv::{FnvHashMap, FnvHashSet};
use std::collections::BTreeSet;

// Use BTreeSet to make a member order robust against addition of new types.
pub fn calculate(
    type_: &Type,
    types: &FnvHashMap<String, Type>,
) -> Result<BTreeSet<Type>, AnalysisError> {
    Ok(
        calculate_canonical(&type_canonicalizer::canonicalize(type_, types)?, types)?
            .into_iter()
            .collect(),
    )
}

fn calculate_canonical(
    type_: &Type,
    types: &FnvHashMap<String, Type>,
) -> Result<FnvHashSet<Type>, AnalysisError> {
    Ok(match type_ {
        Type::Union(union) => calculate_canonical(union.lhs(), types)?
            .union(&calculate_canonical(union.rhs(), types)?)
            .cloned()
            .collect(),
        _ => [type_.clone()].into_iter().collect(),
    })
}
