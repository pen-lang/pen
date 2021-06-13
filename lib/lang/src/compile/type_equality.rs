use super::{type_resolution, CompileError};
use crate::{compile::type_canonicalization, types::Type};
use std::collections::HashMap;

pub fn check_equality(
    one: &Type,
    other: &Type,
    types: &HashMap<String, Type>,
) -> Result<bool, CompileError> {
    let one =
        type_canonicalization::canonicalize(&type_resolution::resolve_type(one, types)?, types)?;
    let other =
        type_canonicalization::canonicalize(&type_resolution::resolve_type(other, types)?, types)?;

    Ok(match (&one, &other) {
        (Type::Function(one), Type::Function(other)) => {
            one.arguments().len() == other.arguments().len()
                && one
                    .arguments()
                    .iter()
                    .zip(other.arguments())
                    .map(|(one, other)| check_equality(one, other, types))
                    .collect::<Result<Vec<_>, _>>()?
                    .iter()
                    .all(|&ok| ok)
                && check_equality(one.result(), other.result(), types)?
        }
        (Type::List(one), Type::List(other)) => {
            check_equality(one.element(), other.element(), types)?
        }
        (Type::Union(one), Type::Union(other)) => {
            check_equality(one.lhs(), other.lhs(), types)?
                && check_equality(one.rhs(), other.rhs(), types)?
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
    use super::*;
    use crate::position::Position;
    use crate::types;

    #[test]
    fn check_numbers() {
        assert!(check_equality(
            &types::Number::new(Position::dummy()).into(),
            &types::Number::new(Position::dummy()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn fail_to_check_number_and_none() {
        assert!(!check_equality(
            &types::Number::new(Position::dummy()).into(),
            &types::None::new(Position::dummy()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_lists() {
        assert!(check_equality(
            &types::List::new(types::Number::new(Position::dummy()), Position::dummy()).into(),
            &types::List::new(types::Number::new(Position::dummy()), Position::dummy()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_functions() {
        assert!(check_equality(
            &types::Function::new(
                vec![],
                types::Number::new(Position::dummy()),
                Position::dummy()
            )
            .into(),
            &types::Function::new(
                vec![],
                types::Number::new(Position::dummy()),
                Position::dummy()
            )
            .into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_function_arguments() {
        assert!(check_equality(
            &types::Function::new(
                vec![],
                types::Number::new(Position::dummy()),
                Position::dummy()
            )
            .into(),
            &types::Function::new(
                vec![],
                types::Number::new(Position::dummy()),
                Position::dummy()
            )
            .into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_union_and_number() {
        assert!(check_equality(
            &types::Union::new(
                types::Number::new(Position::dummy()),
                types::Number::new(Position::dummy()),
                Position::dummy(),
            )
            .into(),
            &types::Number::new(Position::dummy()).into(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_unions() {
        assert!(check_equality(
            &types::Union::new(
                types::Number::new(Position::dummy()),
                types::None::new(Position::dummy()),
                Position::dummy(),
            )
            .into(),
            &types::Union::new(
                types::None::new(Position::dummy()),
                types::Number::new(Position::dummy()),
                Position::dummy(),
            )
            .into(),
            &Default::default(),
        )
        .unwrap());
    }
}
