use super::{super::Type, type_canonicalizer, type_equality_checker, type_resolver, TypeError};
use std::collections::HashMap;

pub fn check(lower: &Type, upper: &Type, types: &HashMap<String, Type>) -> Result<bool, TypeError> {
    let lower = type_canonicalizer::canonicalize(&type_resolver::resolve(lower, types)?, types)?;
    let upper = type_canonicalizer::canonicalize(&type_resolver::resolve(upper, types)?, types)?;

    Ok(match (&lower, &upper) {
        (_, Type::Any(_)) => true,
        (Type::List(one), Type::List(other)) => check(one.element(), other.element(), types)?,
        (Type::Union(lower), Type::Union(_)) => {
            check(lower.lhs(), &upper, types)? && check(lower.rhs(), &upper, types)?
        }
        (lower, Type::Union(union)) => {
            check(lower, union.lhs(), types)? || check(lower, union.rhs(), types)?
        }
        _ => type_equality_checker::check(&lower, &upper, types)?,
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
            &Default::default()
        )
        .unwrap());
    }

    #[test]
    fn check_number_and_union() {
        assert!(check(
            &Number::new(Position::dummy()).into(),
            &Union::new(
                Number::new(Position::dummy()),
                None::new(Position::dummy()),
                Position::dummy()
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
                Number::new(Position::dummy()),
                Number::new(Position::dummy()),
                Position::dummy()
            )
            .into(),
            &Number::new(Position::dummy()).into(),
            &Default::default()
        )
        .unwrap());
    }
}
