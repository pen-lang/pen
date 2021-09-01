use super::{type_canonicalizer, TypeError};
use crate::types::Type;
use std::collections::HashMap;

pub fn check(one: &Type, other: &Type, types: &HashMap<String, Type>) -> Result<bool, TypeError> {
    check_canonical(
        &type_canonicalizer::canonicalize(one, types)?,
        &type_canonicalizer::canonicalize(other, types)?,
        types,
    )
}

fn check_canonical(
    one: &Type,
    other: &Type,
    types: &HashMap<String, Type>,
) -> Result<bool, TypeError> {
    let check = |one, other| check_canonical(one, other, types);

    Ok(match (&one, &other) {
        (Type::Function(one), Type::Function(other)) => {
            one.arguments().len() == other.arguments().len()
                && one
                    .arguments()
                    .iter()
                    .zip(other.arguments())
                    .map(|(one, other)| check(one, other))
                    .collect::<Result<Vec<_>, _>>()?
                    .iter()
                    .all(|&ok| ok)
                && check(one.result(), other.result())?
        }
        (Type::List(one), Type::List(other)) => check(one.element(), other.element())?,
        (Type::Union(one), Type::Union(other)) => {
            check(one.lhs(), other.lhs())? && check(one.rhs(), other.rhs())?
        }
        (Type::Record(one), Type::Record(other)) => one.name() == other.name(),
        (Type::Any(_), Type::Any(_))
        | (Type::Boolean(_), Type::Boolean(_))
        | (Type::None(_), Type::None(_))
        | (Type::Number(_), Type::Number(_))
        | (Type::String(_), Type::String(_)) => true,
        (Type::Reference(_), _) | (_, Type::Reference(_)) => unreachable!(),
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test, types::*};

    #[test]
    fn check_numbers() {
        assert!(check(
            &Number::new(test::position()).into(),
            &Number::new(test::position()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn fail_to_check_number_and_none() {
        assert!(!check(
            &Number::new(test::position()).into(),
            &None::new(test::position()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_lists() {
        assert!(check(
            &List::new(Number::new(test::position()), test::position()).into(),
            &List::new(Number::new(test::position()), test::position()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_functions() {
        assert!(check(
            &Function::new(vec![], Number::new(test::position()), test::position()).into(),
            &Function::new(vec![], Number::new(test::position()), test::position()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_function_arguments() {
        assert!(check(
            &Function::new(vec![], Number::new(test::position()), test::position()).into(),
            &Function::new(vec![], Number::new(test::position()), test::position()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_union_and_number() {
        assert!(check(
            &Union::new(
                Number::new(test::position()),
                Number::new(test::position()),
                test::position(),
            )
            .into(),
            &Number::new(test::position()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_unions() {
        assert!(check(
            &Union::new(
                Number::new(test::position()),
                None::new(test::position()),
                test::position(),
            )
            .into(),
            &Union::new(
                None::new(test::position()),
                Number::new(test::position()),
                test::position(),
            )
            .into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_records() {
        assert!(!check(
            &Record::new("x", test::position()).into(),
            &Record::new("y", test::position()).into(),
            &Default::default(),
        )
        .unwrap());
    }
}
