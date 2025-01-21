use super::{type_canonicalizer, AnalysisError};
use crate::types::Type;
use fnv::FnvHashMap;

pub fn check(
    one: &Type,
    other: &Type,
    types: &FnvHashMap<String, Type>,
) -> Result<bool, AnalysisError> {
    Ok(check_canonical(
        &type_canonicalizer::canonicalize(one, types)?,
        &type_canonicalizer::canonicalize(other, types)?,
    ))
}

fn check_canonical(one: &Type, other: &Type) -> bool {
    match (&one, &other) {
        (Type::Function(one), Type::Function(other)) => {
            one.arguments().len() == other.arguments().len()
                && one
                    .arguments()
                    .iter()
                    .zip(other.arguments())
                    .all(|(one, other)| check_canonical(one, other))
                && check_canonical(one.result(), other.result())
        }
        (Type::List(one), Type::List(other)) => check_canonical(one.element(), other.element()),
        (Type::Map(one), Type::Map(other)) => {
            check_canonical(one.key(), other.key()) && check_canonical(one.value(), other.value())
        }
        (Type::Union(one), Type::Union(other)) => {
            check_canonical(one.lhs(), other.lhs()) && check_canonical(one.rhs(), other.rhs())
        }
        (Type::Record(one), Type::Record(other)) => one.name() == other.name(),
        (Type::Any(_), Type::Any(_))
        | (Type::Boolean(_), Type::Boolean(_))
        | (Type::Error(_), Type::Error(_))
        | (Type::None(_), Type::None(_))
        | (Type::Number(_), Type::Number(_))
        | (Type::String(_), Type::String(_)) => true,
        (Type::Reference(_), _) | (_, Type::Reference(_)) => unreachable!(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test::RecordFake,
        types::{self, *},
    };
    use position::{test::PositionFake, Position};

    #[test]
    fn check_numbers() {
        assert!(check(
            &Number::new(Position::fake()).into(),
            &Number::new(Position::fake()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn fail_to_check_number_and_none() {
        assert!(!check(
            &Number::new(Position::fake()).into(),
            &None::new(Position::fake()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_lists() {
        assert!(check(
            &List::new(Number::new(Position::fake()), Position::fake()).into(),
            &List::new(Number::new(Position::fake()), Position::fake()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_functions() {
        assert!(check(
            &Function::new(vec![], Number::new(Position::fake()), Position::fake()).into(),
            &Function::new(vec![], Number::new(Position::fake()), Position::fake()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_function_arguments() {
        assert!(check(
            &Function::new(vec![], Number::new(Position::fake()), Position::fake()).into(),
            &Function::new(vec![], Number::new(Position::fake()), Position::fake()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_union_and_number() {
        assert!(check(
            &Union::new(
                Number::new(Position::fake()),
                Number::new(Position::fake()),
                Position::fake(),
            )
            .into(),
            &Number::new(Position::fake()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_unions() {
        assert!(check(
            &Union::new(
                Number::new(Position::fake()),
                None::new(Position::fake()),
                Position::fake(),
            )
            .into(),
            &Union::new(
                None::new(Position::fake()),
                Number::new(Position::fake()),
                Position::fake(),
            )
            .into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_records() {
        assert!(!check(
            &types::Record::fake("x").into(),
            &types::Record::fake("y").into(),
            &Default::default(),
        )
        .unwrap());
    }
}
