use super::{type_canonicalizer, union_type_creator, union_type_member_calculator, TypeError};
use crate::types::Type;
use std::collections::HashMap;

pub fn calculate(
    one: &Type,
    other: &Type,
    types: &HashMap<String, Type>,
) -> Result<Option<Type>, TypeError> {
    let one = type_canonicalizer::canonicalize(one, types)?;
    let other = type_canonicalizer::canonicalize(other, types)?;

    Ok(if other.is_any() {
        None
    } else if one.is_any() {
        Some(one)
    } else {
        union_type_creator::create(
            &union_type_member_calculator::calculate(&one, types)?
                .difference(&union_type_member_calculator::calculate(&other, types)?)
                .cloned()
                .collect::<Vec<_>>(),
            one.position(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{test, types};
    use pretty_assertions::assert_eq;

    #[test]
    fn calculate_with_any_and_any() {
        assert_eq!(
            calculate(
                &types::Any::new(test::position()).into(),
                &types::Any::new(test::position()).into(),
                &Default::default(),
            ),
            Ok(None)
        );
    }

    #[test]
    fn calculate_with_any_and_number() {
        assert_eq!(
            calculate(
                &types::Any::new(test::position()).into(),
                &types::Number::new(test::position()).into(),
                &Default::default(),
            ),
            Ok(Some(types::Any::new(test::position()).into()))
        );
    }

    #[test]
    fn calculate_with_number_and_any() {
        assert_eq!(
            calculate(
                &types::Number::new(test::position()).into(),
                &types::Any::new(test::position()).into(),
                &Default::default(),
            ),
            Ok(None)
        );
    }

    #[test]
    fn calculate_with_number_and_number() {
        assert_eq!(
            calculate(
                &types::Number::new(test::position()).into(),
                &types::Number::new(test::position()).into(),
                &Default::default(),
            ),
            Ok(None)
        );
    }

    #[test]
    fn calculate_with_union_and_number() {
        assert_eq!(
            calculate(
                &types::Union::new(
                    types::Number::new(test::position()),
                    types::None::new(test::position()),
                    test::position()
                )
                .into(),
                &types::Number::new(test::position()).into(),
                &Default::default(),
            ),
            Ok(Some(types::None::new(test::position()).into()))
        );
    }

    #[test]
    fn calculate_with_union_and_union() {
        assert_eq!(
            calculate(
                &types::Union::new(
                    types::Union::new(
                        types::Number::new(test::position()),
                        types::Boolean::new(test::position()),
                        test::position()
                    ),
                    types::None::new(test::position()),
                    test::position()
                )
                .into(),
                &types::Union::new(
                    types::Boolean::new(test::position()),
                    types::None::new(test::position()),
                    test::position()
                )
                .into(),
                &Default::default(),
            ),
            Ok(Some(types::Number::new(test::position()).into()))
        );
    }
}
