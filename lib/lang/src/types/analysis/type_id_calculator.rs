use super::{error::TypeError, type_canonicalizer};
use crate::types::Type;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

pub fn calculate(type_: &Type, types: &HashMap<String, Type>) -> Result<String, TypeError> {
    let mut hasher = DefaultHasher::new();

    calculate_string(&type_canonicalizer::canonicalize(type_, types)?, types)?.hash(&mut hasher);

    Ok(format!("{:x}", hasher.finish()))
}

fn calculate_string(type_: &Type, types: &HashMap<String, Type>) -> Result<String, TypeError> {
    let calculate_string = |type_| calculate_string(type_, types);

    Ok(match type_canonicalizer::canonicalize(type_, types)? {
        Type::Any(_) => "any".into(),
        Type::Boolean(_) => "boolean".into(),
        Type::Function(function) => format!(
            "(\\({}){})",
            function
                .arguments()
                .iter()
                .map(|argument| calculate_string(argument))
                .collect::<Result<Vec<_>, _>>()?
                .join(","),
            calculate_string(function.result())?
        ),
        Type::List(list) => format!("[{}]", calculate_string(list.element())?),
        Type::None(_) => "none".into(),
        Type::Number(_) => "number".into(),
        Type::Record(record) => record.name().into(),
        Type::String(_) => "string".into(),
        Type::Union(union) => format!(
            "({}|{})",
            calculate_string(union.lhs())?,
            calculate_string(union.rhs())?,
        ),
        Type::Reference(_) => unreachable!(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{position::Position, types};

    #[test]
    fn calculate_none_list_type_id() {
        assert_eq!(
            calculate_string(
                &types::List::new(types::None::new(Position::dummy()), Position::dummy()).into(),
                &Default::default()
            ),
            Ok("[none]".into())
        );
    }

    #[test]
    fn calculate_any_list_type_id() {
        assert_eq!(
            calculate_string(
                &types::List::new(types::Any::new(Position::dummy()), Position::dummy()).into(),
                &Default::default(),
            ),
            Ok("[any]".into())
        );
    }

    #[test]
    fn calculate_union_list_type_id() {
        assert_eq!(
            calculate_string(
                &types::List::new(
                    types::Union::new(
                        types::Number::new(Position::dummy()),
                        types::None::new(Position::dummy()),
                        Position::dummy()
                    ),
                    Position::dummy()
                )
                .into(),
                &Default::default(),
            ),
            Ok("[(none|number)]".into())
        );
    }

    #[test]
    fn canonicalize_types_before_id_calculation() {
        assert_eq!(
            calculate(
                &types::Union::new(
                    types::Number::new(Position::dummy()),
                    types::None::new(Position::dummy()),
                    Position::dummy()
                )
                .into(),
                &Default::default(),
            ),
            calculate(
                &types::Union::new(
                    types::None::new(Position::dummy()),
                    types::Number::new(Position::dummy()),
                    Position::dummy()
                )
                .into(),
                &Default::default(),
            )
        );
    }
}
