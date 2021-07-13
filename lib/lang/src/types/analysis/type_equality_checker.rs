use super::{super::Type, type_canonicalizer, TypeError};
use std::collections::HashMap;

pub fn check(one: &Type, other: &Type, types: &HashMap<String, Type>) -> Result<bool, TypeError> {
    let one = type_canonicalizer::canonicalize(one, types)?;
    let other = type_canonicalizer::canonicalize(other, types)?;

    Ok(match (&one, &other) {
        (Type::Function(one), Type::Function(other)) => {
            one.arguments().len() == other.arguments().len()
                && one
                    .arguments()
                    .iter()
                    .zip(other.arguments())
                    .map(|(one, other)| check(one, other, types))
                    .collect::<Result<Vec<_>, _>>()?
                    .iter()
                    .all(|&ok| ok)
                && check(one.result(), other.result(), types)?
        }
        (Type::List(one), Type::List(other)) => check(one.element(), other.element(), types)?,
        (Type::Union(one), Type::Union(other)) => {
            check(one.lhs(), other.lhs(), types)? && check(one.rhs(), other.rhs(), types)?
        }
        (Type::Any(_), Type::Any(_))
        | (Type::Boolean(_), Type::Boolean(_))
        | (Type::None(_), Type::None(_))
        | (Type::Number(_), Type::Number(_))
        | (Type::Record(_), Type::Record(_))
        | (Type::String(_), Type::String(_)) => true,
        (Type::Reference(_), _) | (_, Type::Reference(_)) => unreachable!(),
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::{super::super::*, *};
    use crate::position::Position;

    #[test]
    fn check_numbers() {
        assert!(check(
            &Number::new(Position::dummy()).into(),
            &Number::new(Position::dummy()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn fail_to_check_number_and_none() {
        assert!(!check(
            &Number::new(Position::dummy()).into(),
            &None::new(Position::dummy()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_lists() {
        assert!(check(
            &List::new(Number::new(Position::dummy()), Position::dummy()).into(),
            &List::new(Number::new(Position::dummy()), Position::dummy()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_functions() {
        assert!(check(
            &Function::new(vec![], Number::new(Position::dummy()), Position::dummy()).into(),
            &Function::new(vec![], Number::new(Position::dummy()), Position::dummy()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_function_arguments() {
        assert!(check(
            &Function::new(vec![], Number::new(Position::dummy()), Position::dummy()).into(),
            &Function::new(vec![], Number::new(Position::dummy()), Position::dummy()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_union_and_number() {
        assert!(check(
            &Union::new(
                Number::new(Position::dummy()),
                Number::new(Position::dummy()),
                Position::dummy(),
            )
            .into(),
            &Number::new(Position::dummy()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_unions() {
        assert!(check(
            &Union::new(
                Number::new(Position::dummy()),
                None::new(Position::dummy()),
                Position::dummy(),
            )
            .into(),
            &Union::new(
                None::new(Position::dummy()),
                Number::new(Position::dummy()),
                Position::dummy(),
            )
            .into(),
            &Default::default(),
        )
        .unwrap());
    }
}
