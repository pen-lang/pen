use super::{type_resolution, CompileError};
use crate::types::{self, Type};
use std::collections::{BTreeSet, HashMap};

pub fn canonicalize_type(
    type_: &Type,
    types: &HashMap<String, Type>,
) -> Result<Type, CompileError> {
    Ok(match type_resolution::resolve_type(type_, types)? {
        Type::Function(function) => types::Function::new(
            function
                .arguments()
                .iter()
                .map(|type_| canonicalize_type(type_, types))
                .collect::<Result<_, _>>()?,
            canonicalize_type(function.result(), types)?,
            function.position().clone(),
        )
        .into(),
        Type::List(list) => types::List::new(
            canonicalize_type(list.element(), types)?,
            list.position().clone(),
        )
        .into(),
        Type::Union(union) => canonicalize_union(&union, types)?,
        Type::Any(_)
        | Type::Boolean(_)
        | Type::Record(_)
        | Type::None(_)
        | Type::Number(_)
        | Type::String(_) => type_.clone(),
        Type::Reference(_) => unreachable!(),
    })
}

fn canonicalize_union(
    union: &types::Union,
    types: &HashMap<String, Type>,
) -> Result<Type, CompileError> {
    Ok(collect_types(&union.clone().into(), types)?
        .into_iter()
        .reduce(|one, other| types::Union::new(one, other, union.position().clone()).into())
        .unwrap())
}

fn collect_types(
    type_: &Type,
    types: &HashMap<String, Type>,
) -> Result<BTreeSet<Type>, CompileError> {
    Ok(match type_resolution::resolve_type(type_, types)? {
        Type::Union(union) => collect_types(union.lhs(), types)?
            .into_iter()
            .chain(collect_types(union.rhs(), types)?)
            .collect(),
        Type::Any(_)
        | Type::Boolean(_)
        | Type::Function(_)
        | Type::Record(_)
        | Type::List(_)
        | Type::None(_)
        | Type::Number(_)
        | Type::String(_) => vec![canonicalize_type(type_, types)?].into_iter().collect(),
        Type::Reference(_) => unreachable!(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;

    #[test]
    fn canonicalize_number() {
        assert_eq!(
            canonicalize_type(
                &types::Number::new(Position::dummy()).into(),
                &Default::default(),
            ),
            Ok(types::Number::new(Position::dummy()).into())
        );
    }
}
