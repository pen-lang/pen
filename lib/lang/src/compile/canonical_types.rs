use super::{type_context::TypeContext, type_resolution, CompileError};
use crate::types::{self, Type};
use std::collections::BTreeSet;

pub fn canonicalize_type(type_: &Type, context: &TypeContext) -> Result<Type, CompileError> {
    Ok(match type_resolution::resolve_type(type_, context)? {
        Type::Function(function) => types::Function::new(
            function
                .arguments()
                .iter()
                .map(|type_| canonicalize_type(type_, context))
                .collect::<Result<_, _>>()?,
            canonicalize_type(function.result(), context)?,
            function.position().clone(),
        )
        .into(),
        Type::List(list) => types::List::new(
            canonicalize_type(list.element(), context)?,
            list.position().clone(),
        )
        .into(),
        Type::Union(union) => canonicalize_union(&union, context)?,
        Type::Any(_)
        | Type::Boolean(_)
        | Type::Record(_)
        | Type::None(_)
        | Type::Number(_)
        | Type::String(_) => type_.clone(),
        Type::Reference(_) => unreachable!(),
    })
}

fn canonicalize_union(union: &types::Union, context: &TypeContext) -> Result<Type, CompileError> {
    Ok(collect_types(&union.clone().into(), context)?
        .into_iter()
        .reduce(|one, other| types::Union::new(one, other, union.position().clone()).into())
        .unwrap())
}

fn collect_types(type_: &Type, context: &TypeContext) -> Result<BTreeSet<Type>, CompileError> {
    Ok(match type_resolution::resolve_type(type_, context)? {
        Type::Union(union) => collect_types(union.lhs(), context)?
            .into_iter()
            .chain(collect_types(union.rhs(), context)?)
            .collect(),
        Type::Any(_)
        | Type::Boolean(_)
        | Type::Function(_)
        | Type::Record(_)
        | Type::List(_)
        | Type::None(_)
        | Type::Number(_)
        | Type::String(_) => vec![canonicalize_type(type_, context)?]
            .into_iter()
            .collect(),
        Type::Reference(_) => unreachable!(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hir::Module, position::Position};

    #[test]
    fn canonicalize_number() {
        assert_eq!(
            canonicalize_type(
                &types::Number::new(Position::dummy()).into(),
                &TypeContext::new(&Module::new(vec![], vec![], vec![], vec![]))
            ),
            Ok(types::Number::new(Position::dummy()).into())
        );
    }
}
