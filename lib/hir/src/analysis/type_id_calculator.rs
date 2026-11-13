use super::{error::AnalysisError, type_canonicalizer};
use crate::types::Type;
use fnv::FnvHashMap;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub fn calculate(type_: &Type, types: &FnvHashMap<String, Type>) -> Result<String, AnalysisError> {
    let mut hasher = DefaultHasher::new();

    calculate_canonical_string(type_, types)?.hash(&mut hasher);

    Ok(format!("{:x}", hasher.finish()))
}

fn calculate_canonical_string(
    type_: &Type,
    types: &FnvHashMap<String, Type>,
) -> Result<String, AnalysisError> {
    Ok(calculate_string(&type_canonicalizer::canonicalize(
        type_, types,
    )?))
}

fn calculate_string(type_: &Type) -> String {
    match type_ {
        Type::Any(_) => "any".into(),
        Type::Boolean(_) => "boolean".into(),
        Type::Error(_) => "error".into(),
        Type::Function(function) => format!(
            "(\\({}){})",
            function
                .arguments()
                .iter()
                .map(calculate_string)
                .collect::<Vec<_>>()
                .join(","),
            calculate_string(function.result())
        ),
        Type::List(list) => format!("[{}]", calculate_string(list.element())),
        Type::Map(map) => format!(
            "{{{}:{}}}",
            calculate_string(map.key()),
            calculate_string(map.value())
        ),
        Type::None(_) => "none".into(),
        Type::Number(_) => "number".into(),
        Type::Record(record) => format!("record({})", record.name()),
        Type::String(_) => "string".into(),
        Type::Union(union) => format!(
            "({}|{})",
            calculate_string(union.lhs()),
            calculate_string(union.rhs()),
        ),
        Type::Reference(_) => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types;
    use position::{Position, test::PositionFake};

    #[test]
    fn calculate_none_list_type_id() {
        assert_eq!(
            calculate_canonical_string(
                &types::List::new(types::None::new(Position::fake()), Position::fake()).into(),
                &Default::default()
            ),
            Ok("[none]".into())
        );
    }

    #[test]
    fn calculate_any_list_type_id() {
        assert_eq!(
            calculate_canonical_string(
                &types::List::new(types::Any::new(Position::fake()), Position::fake()).into(),
                &Default::default(),
            ),
            Ok("[any]".into())
        );
    }

    #[test]
    fn calculate_union_list_type_id() {
        assert_eq!(
            calculate_canonical_string(
                &types::List::new(
                    types::Union::new(
                        types::Number::new(Position::fake()),
                        types::None::new(Position::fake()),
                        Position::fake()
                    ),
                    Position::fake()
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
                    types::Number::new(Position::fake()),
                    types::None::new(Position::fake()),
                    Position::fake()
                )
                .into(),
                &Default::default(),
            ),
            calculate(
                &types::Union::new(
                    types::None::new(Position::fake()),
                    types::Number::new(Position::fake()),
                    Position::fake()
                )
                .into(),
                &Default::default(),
            )
        );
    }
}
