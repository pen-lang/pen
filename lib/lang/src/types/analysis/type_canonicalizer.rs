use super::{type_resolver, TypeAnalysisError};
use crate::types::{self, Type};
use std::collections::{BTreeSet, HashMap};

pub fn canonicalize(type_: &Type, types: &HashMap<String, Type>) -> Result<Type, TypeAnalysisError> {
    Ok(match type_resolver::resolve_type(type_, types)? {
        Type::Function(function) => types::Function::new(
            function
                .arguments()
                .iter()
                .map(|type_| canonicalize(type_, types))
                .collect::<Result<_, _>>()?,
            canonicalize(function.result(), types)?,
            function.position().clone(),
        )
        .into(),
        Type::List(list) => types::List::new(
            canonicalize(list.element(), types)?,
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
) -> Result<Type, TypeAnalysisError> {
    Ok(collect_types(&union.clone().into(), types)?
        .into_iter()
        .reduce(|one, other| types::Union::new(one, other, union.position().clone()).into())
        .unwrap())
}

fn collect_types(type_: &Type, types: &HashMap<String, Type>) -> Result<BTreeSet<Type>, TypeAnalysisError> {
    Ok(match type_resolver::resolve_type(type_, types)? {
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
        | Type::String(_) => vec![canonicalize(type_, types)?].into_iter().collect(),
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
            canonicalize(
                &types::Number::new(Position::dummy()).into(),
                &Default::default(),
            ),
            Ok(types::Number::new(Position::dummy()).into())
        );
    }

    #[test]
    fn canonicalize_union_of_numbers() {
        assert_eq!(
            canonicalize(
                &types::Union::new(
                    types::Number::new(Position::dummy()),
                    types::Number::new(Position::dummy()),
                    Position::dummy()
                )
                .into(),
                &Default::default(),
            ),
            Ok(types::Number::new(Position::dummy()).into())
        );
    }

    #[test]
    fn canonicalize_union_of_3_types() {
        assert_eq!(
            canonicalize(
                &types::Union::new(
                    types::Number::new(Position::dummy()),
                    types::Union::new(
                        types::Boolean::new(Position::dummy()),
                        types::None::new(Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy()
                )
                .into(),
                &Default::default(),
            ),
            Ok(types::Union::new(
                types::Union::new(
                    types::Boolean::new(Position::dummy()),
                    types::None::new(Position::dummy()),
                    Position::dummy()
                ),
                types::Number::new(Position::dummy()),
                Position::dummy()
            )
            .into())
        );
    }

    #[test]
    fn canonicalize_union_of_function_argument() {
        assert_eq!(
            canonicalize(
                &types::Function::new(
                    vec![types::Union::new(
                        types::Number::new(Position::dummy()),
                        types::Number::new(Position::dummy()),
                        Position::dummy()
                    )
                    .into()],
                    types::None::new(Position::dummy()),
                    Position::dummy(),
                )
                .into(),
                &Default::default(),
            ),
            Ok(types::Function::new(
                vec![types::Number::new(Position::dummy()).into()],
                types::None::new(Position::dummy()),
                Position::dummy(),
            )
            .into())
        );
    }

    #[test]
    fn canonicalize_union_of_function_result() {
        assert_eq!(
            canonicalize(
                &types::Function::new(
                    vec![],
                    types::Union::new(
                        types::Number::new(Position::dummy()),
                        types::Number::new(Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy(),
                )
                .into(),
                &Default::default(),
            ),
            Ok(types::Function::new(
                vec![],
                types::Number::new(Position::dummy()),
                Position::dummy(),
            )
            .into())
        );
    }

    #[test]
    fn canonicalize_union_of_list_element() {
        assert_eq!(
            canonicalize(
                &types::List::new(
                    types::Union::new(
                        types::Number::new(Position::dummy()),
                        types::Number::new(Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy(),
                )
                .into(),
                &Default::default(),
            ),
            Ok(types::List::new(types::Number::new(Position::dummy()), Position::dummy(),).into())
        );
    }
}
