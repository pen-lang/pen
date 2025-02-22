use super::{AnalysisError, type_canonicalizer, union_type_creator, union_type_member_calculator};
use crate::types::Type;
use fnv::FnvHashMap;

pub fn calculate(
    one: &Type,
    other: &Type,
    types: &FnvHashMap<String, Type>,
) -> Result<Option<Type>, AnalysisError> {
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
    use crate::types;
    use position::{Position, test::PositionFake};
    use pretty_assertions::assert_eq;

    #[test]
    fn calculate_with_any_and_any() {
        assert_eq!(
            calculate(
                &types::Any::new(Position::fake()).into(),
                &types::Any::new(Position::fake()).into(),
                &Default::default(),
            ),
            Ok(None)
        );
    }

    #[test]
    fn calculate_with_any_and_number() {
        assert_eq!(
            calculate(
                &types::Any::new(Position::fake()).into(),
                &types::Number::new(Position::fake()).into(),
                &Default::default(),
            ),
            Ok(Some(types::Any::new(Position::fake()).into()))
        );
    }

    #[test]
    fn calculate_with_number_and_any() {
        assert_eq!(
            calculate(
                &types::Number::new(Position::fake()).into(),
                &types::Any::new(Position::fake()).into(),
                &Default::default(),
            ),
            Ok(None)
        );
    }

    #[test]
    fn calculate_with_number_and_number() {
        assert_eq!(
            calculate(
                &types::Number::new(Position::fake()).into(),
                &types::Number::new(Position::fake()).into(),
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
                    types::Number::new(Position::fake()),
                    types::None::new(Position::fake()),
                    Position::fake()
                )
                .into(),
                &types::Number::new(Position::fake()).into(),
                &Default::default(),
            ),
            Ok(Some(types::None::new(Position::fake()).into()))
        );
    }

    #[test]
    fn calculate_with_union_and_union() {
        assert_eq!(
            calculate(
                &types::Union::new(
                    types::Union::new(
                        types::Number::new(Position::fake()),
                        types::Boolean::new(Position::fake()),
                        Position::fake()
                    ),
                    types::None::new(Position::fake()),
                    Position::fake()
                )
                .into(),
                &types::Union::new(
                    types::Boolean::new(Position::fake()),
                    types::None::new(Position::fake()),
                    Position::fake()
                )
                .into(),
                &Default::default(),
            ),
            Ok(Some(types::Number::new(Position::fake()).into()))
        );
    }
}
