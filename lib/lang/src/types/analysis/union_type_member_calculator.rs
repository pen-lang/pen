
use super::{type_canonicalizer, TypeError};
use crate::types::Type;
use std::collections::{BTreeSet, HashMap};

pub fn calculate(type_: &Type, types: &HashMap<String, Type>) -> Result<BTreeSet<Type>, TypeError> {
    calculate_canonical(&type_canonicalizer::canonicalize(type_, types)?, types)
}

fn calculate_canonical(
    type_: &Type,
    types: &HashMap<String, Type>,
) -> Result<BTreeSet<Type>, TypeError> {
    Ok(match type_ {
        Type::Union(union) => calculate_canonical(union.lhs(), types)?
            .union(&calculate_canonical(union.rhs(), types)?)
            .cloned()
            .collect(),
        _ => vec![type_.clone()].into_iter().collect(),
    })
}
