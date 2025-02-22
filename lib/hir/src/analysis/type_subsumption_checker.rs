use super::{AnalysisError, type_canonicalizer, type_equality_checker};
use crate::types::Type;
use fnv::FnvHashMap;

pub fn check(
    lower: &Type,
    upper: &Type,
    types: &FnvHashMap<String, Type>,
) -> Result<bool, AnalysisError> {
    check_canonical(
        &type_canonicalizer::canonicalize(lower, types)?,
        &type_canonicalizer::canonicalize(upper, types)?,
        types,
    )
}

fn check_canonical(
    lower: &Type,
    upper: &Type,
    types: &FnvHashMap<String, Type>,
) -> Result<bool, AnalysisError> {
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
    use super::*;
    use crate::types::*;
    use position::{Position, test::PositionFake};

    #[test]
    fn check_numbers() {
        assert!(
            check(
                &Number::new(Position::fake()).into(),
                &Number::new(Position::fake()).into(),
                &Default::default()
            )
            .unwrap()
        );
    }

    #[test]
    fn check_number_and_union() {
        assert!(
            check(
                &Number::new(Position::fake()).into(),
                &Union::new(
                    Number::new(Position::fake()),
                    None::new(Position::fake()),
                    Position::fake()
                )
                .into(),
                &Default::default()
            )
            .unwrap()
        );
    }

    #[test]
    fn check_non_canonical_union_and_number() {
        assert!(
            check(
                &Union::new(
                    Number::new(Position::fake()),
                    Number::new(Position::fake()),
                    Position::fake()
                )
                .into(),
                &Number::new(Position::fake()).into(),
                &Default::default()
            )
            .unwrap()
        );
    }

    #[test]
    fn check_lists() {
        assert!(
            check(
                &List::new(Number::new(Position::fake()), Position::fake()).into(),
                &List::new(Number::new(Position::fake()), Position::fake()).into(),
                &Default::default()
            )
            .unwrap()
        );
    }

    #[test]
    fn fail_to_check_lists_with_covariance() {
        assert_eq!(
            check(
                &List::new(Number::new(Position::fake()), Position::fake()).into(),
                &List::new(
                    Union::new(
                        Number::new(Position::fake()),
                        None::new(Position::fake()),
                        Position::fake()
                    ),
                    Position::fake()
                )
                .into(),
                &Default::default()
            ),
            Ok(false)
        );
    }
}
