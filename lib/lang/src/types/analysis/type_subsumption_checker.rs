use super::{super::Type, type_canonicalizer, type_equality_checker, TypeError};
use std::collections::HashMap;

pub fn check(lower: &Type, upper: &Type, types: &HashMap<String, Type>) -> Result<bool, TypeError> {
    check_canonical(
        &type_canonicalizer::canonicalize(lower, types)?,
        &type_canonicalizer::canonicalize(upper, types)?,
        types,
    )
}

fn check_canonical(
    lower: &Type,
    upper: &Type,
    types: &HashMap<String, Type>,
) -> Result<bool, TypeError> {
    let check = |lower, upper| check_canonical(lower, upper, types);

    Ok(match (&lower, &upper) {
        (_, Type::Any(_)) => true,
        (Type::Union(lower), Type::Union(_)) => {
            check(lower.lhs(), upper)? && check(lower.rhs(), upper)?
        }
        (lower, Type::Union(union)) => check(lower, union.lhs())? || check(lower, union.rhs())?,
        _ => type_equality_checker::check(lower, upper, types)?,
    })
}

#[cfg(test)]
mod tests {
    use super::{super::super::*, *};
    use crate::test;

    #[test]
    fn check_numbers() {
        assert!(check(
            &Number::new(test::position()).into(),
            &Number::new(test::position()).into(),
            &Default::default()
        )
        .unwrap());
    }

    #[test]
    fn check_number_and_union() {
        assert!(check(
            &Number::new(test::position()).into(),
            &Union::new(
                Number::new(test::position()),
                None::new(test::position()),
                test::position()
            )
            .into(),
            &Default::default()
        )
        .unwrap());
    }

    #[test]
    fn check_non_canonical_union_and_number() {
        assert!(check(
            &Union::new(
                Number::new(test::position()),
                Number::new(test::position()),
                test::position()
            )
            .into(),
            &Number::new(test::position()).into(),
            &Default::default()
        )
        .unwrap());
    }

    #[test]
    fn check_lists() {
        assert!(check(
            &List::new(Number::new(test::position()), test::position()).into(),
            &List::new(Number::new(test::position()), test::position()).into(),
            &Default::default()
        )
        .unwrap());
    }

    #[test]
    fn fail_to_check_lists_with_covariance() {
        assert_eq!(
            check(
                &List::new(Number::new(test::position()), test::position()).into(),
                &List::new(
                    Union::new(
                        Number::new(test::position()),
                        None::new(test::position()),
                        test::position()
                    ),
                    test::position()
                )
                .into(),
                &Default::default()
            ),
            Ok(false)
        );
    }
}
