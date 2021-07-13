use super::{type_canonicalizer, union_type_member_calculator, TypeError};
use crate::types::Type;
use std::collections::{HashMap, HashSet};

pub fn calculate(
    one: &Type,
    other: &Type,
    types: &HashMap<String, Type>,
) -> Result<Option<HashSet<Type>>, TypeError> {
    let one = type_canonicalizer::canonicalize(one, types)?;
    let other = type_canonicalizer::canonicalize(other, types)?;

    Ok(if one.is_any() {
        None
    } else if other.is_any() {
        Some(Default::default())
    } else {
        Some(
            union_type_member_calculator::calculate(&one, types)?
                .difference(&union_type_member_calculator::calculate(&other, types)?)
                .cloned()
                .collect(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{position::Position, types};
    use pretty_assertions::assert_eq;

    #[test]
    fn calculate_with_any_and_any() {
        assert_eq!(
            calculate(
                &types::Any::new(Position::dummy()).into(),
                &types::Any::new(Position::dummy()).into(),
                &Default::default(),
            ),
            Ok(None)
        );
    }

    #[test]
    fn calculate_with_any_and_number() {
        assert_eq!(
            calculate(
                &types::Any::new(Position::dummy()).into(),
                &types::Number::new(Position::dummy()).into(),
                &Default::default(),
            ),
            Ok(None)
        );
    }

    #[test]
    fn calculate_with_number_and_any() {
        assert_eq!(
            calculate(
                &types::Number::new(Position::dummy()).into(),
                &types::Any::new(Position::dummy()).into(),
                &Default::default(),
            ),
            Ok(Some(Default::default()))
        );
    }

    #[test]
    fn calculate_with_number_and_number() {
        assert_eq!(
            calculate(
                &types::Number::new(Position::dummy()).into(),
                &types::Number::new(Position::dummy()).into(),
                &Default::default(),
            ),
            Ok(Some(Default::default()))
        );
    }

    #[test]
    fn calculate_with_union_and_number() {
        assert_eq!(
            calculate(
                &types::Union::new(
                    types::Number::new(Position::dummy()),
                    types::None::new(Position::dummy()),
                    Position::dummy()
                )
                .into(),
                &types::Number::new(Position::dummy()).into(),
                &Default::default(),
            ),
            Ok(Some(
                vec![types::None::new(Position::dummy()).into()]
                    .into_iter()
                    .collect()
            ))
        );
    }

    #[test]
    fn calculate_with_union_and_union() {
        assert_eq!(
            calculate(
                &types::Union::new(
                    types::Union::new(
                        types::Number::new(Position::dummy()),
                        types::Boolean::new(Position::dummy()),
                        Position::dummy()
                    ),
                    types::None::new(Position::dummy()),
                    Position::dummy()
                )
                .into(),
                &types::Union::new(
                    types::Boolean::new(Position::dummy()),
                    types::None::new(Position::dummy()),
                    Position::dummy()
                )
                .into(),
                &Default::default(),
            ),
            Ok(Some(
                vec![types::Number::new(Position::dummy()).into()]
                    .into_iter()
                    .collect()
            ))
        );
    }
}
