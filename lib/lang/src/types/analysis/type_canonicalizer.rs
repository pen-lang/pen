use super::{super::*, type_resolver, TypeError};
use std::collections::{BTreeSet, HashMap};

pub fn canonicalize(type_: &Type, types: &HashMap<String, Type>) -> Result<Type, TypeError> {
    let type_ = type_resolver::resolve(type_, types)?;

    Ok(match &type_ {
        Type::Function(function) => Function::new(
            function
                .arguments()
                .iter()
                .map(|type_| canonicalize(type_, types))
                .collect::<Result<_, _>>()?,
            canonicalize(function.result(), types)?,
            function.position().clone(),
        )
        .into(),
        Type::List(list) => List::new(
            canonicalize(list.element(), types)?,
            list.position().clone(),
        )
        .into(),
        Type::Union(union) => canonicalize_union(union, types)?,
        Type::Any(_)
        | Type::Boolean(_)
        | Type::Record(_)
        | Type::None(_)
        | Type::Number(_)
        | Type::String(_) => type_.clone(),
        Type::Reference(_) => unreachable!(),
    })
}

pub fn canonicalize_function(
    type_: &Type,
    types: &HashMap<String, Type>,
) -> Result<Option<Function>, TypeError> {
    Ok(match canonicalize(type_, types)? {
        Type::Function(function) => Some(function),
        _ => None,
    })
}

pub fn canonicalize_list(
    type_: &Type,
    types: &HashMap<String, Type>,
) -> Result<Option<List>, TypeError> {
    Ok(match canonicalize(type_, types)? {
        Type::List(list) => Some(list),
        _ => None,
    })
}

pub fn canonicalize_record(
    type_: &Type,
    types: &HashMap<String, Type>,
) -> Result<Option<Record>, TypeError> {
    Ok(match canonicalize(type_, types)? {
        Type::Record(record) => Some(record),
        _ => None,
    })
}

fn canonicalize_union(union: &Union, types: &HashMap<String, Type>) -> Result<Type, TypeError> {
    Ok(collect_types(&union.clone().into(), types)?
        .into_iter()
        .reduce(|one, other| {
            if one.is_any() {
                one
            } else if other.is_any() {
                other
            } else {
                Union::new(one, other, union.position().clone()).into()
            }
        })
        .unwrap())
}

fn collect_types(type_: &Type, types: &HashMap<String, Type>) -> Result<BTreeSet<Type>, TypeError> {
    Ok(match type_resolver::resolve(type_, types)? {
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
            canonicalize(&Number::new(Position::dummy()).into(), &Default::default(),),
            Ok(Number::new(Position::dummy()).into())
        );
    }

    #[test]
    fn canonicalize_union_of_numbers() {
        assert_eq!(
            canonicalize(
                &Union::new(
                    Number::new(Position::dummy()),
                    Number::new(Position::dummy()),
                    Position::dummy()
                )
                .into(),
                &Default::default(),
            ),
            Ok(Number::new(Position::dummy()).into())
        );
    }

    #[test]
    fn canonicalize_union_of_3_types() {
        assert_eq!(
            canonicalize(
                &Union::new(
                    Number::new(Position::dummy()),
                    Union::new(
                        Boolean::new(Position::dummy()),
                        None::new(Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy()
                )
                .into(),
                &Default::default(),
            ),
            Ok(Union::new(
                Union::new(
                    Boolean::new(Position::dummy()),
                    None::new(Position::dummy()),
                    Position::dummy()
                ),
                Number::new(Position::dummy()),
                Position::dummy()
            )
            .into())
        );
    }

    #[test]
    fn canonicalize_union_of_function_argument() {
        assert_eq!(
            canonicalize(
                &Function::new(
                    vec![Union::new(
                        Number::new(Position::dummy()),
                        Number::new(Position::dummy()),
                        Position::dummy()
                    )
                    .into()],
                    None::new(Position::dummy()),
                    Position::dummy(),
                )
                .into(),
                &Default::default(),
            ),
            Ok(Function::new(
                vec![Number::new(Position::dummy()).into()],
                None::new(Position::dummy()),
                Position::dummy(),
            )
            .into())
        );
    }

    #[test]
    fn canonicalize_union_of_function_result() {
        assert_eq!(
            canonicalize(
                &Function::new(
                    vec![],
                    Union::new(
                        Number::new(Position::dummy()),
                        Number::new(Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy(),
                )
                .into(),
                &Default::default(),
            ),
            Ok(Function::new(vec![], Number::new(Position::dummy()), Position::dummy(),).into())
        );
    }

    #[test]
    fn canonicalize_union_of_list_element() {
        assert_eq!(
            canonicalize(
                &List::new(
                    Union::new(
                        Number::new(Position::dummy()),
                        Number::new(Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy(),
                )
                .into(),
                &Default::default(),
            ),
            Ok(List::new(Number::new(Position::dummy()), Position::dummy(),).into())
        );
    }

    #[test]
    fn canonicalize_union_with_any() {
        assert_eq!(
            canonicalize(
                &Union::new(
                    Number::new(Position::dummy()),
                    Any::new(Position::dummy()),
                    Position::dummy()
                )
                .into(),
                &Default::default(),
            ),
            Ok(Any::new(Position::dummy()).into())
        );
    }

    #[test]
    fn canonicalize_reference() {
        assert_eq!(
            canonicalize(
                &Reference::new("t", Position::dummy()).into(),
                &vec![("t".into(), Number::new(Position::dummy()).into())]
                    .into_iter()
                    .collect(),
            ),
            Ok(Number::new(Position::dummy()).into())
        );
    }
}
