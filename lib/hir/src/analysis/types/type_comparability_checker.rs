use super::{record_field_resolver, type_resolver, TypeError};
use crate::types::{RecordField, Type};
use fnv::{FnvHashMap, FnvHashSet};

pub fn check(
    type_: &Type,
    types: &FnvHashMap<String, Type>,
    record_types: &FnvHashMap<String, Vec<RecordField>>,
) -> Result<bool, TypeError> {
    check_with_cache(type_, &Default::default(), types, record_types)
}

fn check_with_cache(
    type_: &Type,
    record_names: &FnvHashSet<String>,
    types: &FnvHashMap<String, Type>,
    record_types: &FnvHashMap<String, Vec<RecordField>>,
) -> Result<bool, TypeError> {
    let check_with_cache =
        |type_, record_names| check_with_cache(type_, record_names, types, record_types);

    Ok(match type_ {
        Type::Any(_) => false,
        Type::Boolean(_) => true,
        Type::Function(_) => false,
        Type::List(list) => check_with_cache(list.element(), record_names)?,
        Type::None(_) => true,
        Type::Number(_) => true,
        Type::Record(record) => {
            if record_names.contains(record.name()) {
                true
            } else {
                let record_names = record_names
                    .clone()
                    .into_iter()
                    .chain([record.name().into()])
                    .collect();

                record_field_resolver::resolve(type_, type_.position(), types, record_types)?
                    .iter()
                    .map(|field| check_with_cache(field.type_(), &record_names))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .all(|flag| flag)
            }
        }
        Type::Reference(reference) => {
            check_with_cache(&type_resolver::resolve(reference, types)?, record_names)?
        }
        Type::String(_) => true,
        Type::Union(union) => {
            check_with_cache(union.lhs(), record_names)?
                && check_with_cache(union.rhs(), record_names)?
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types;
    use position::{test::PositionFake, Position};

    #[test]
    fn check_record_type() {
        assert!(check(
            &types::Record::new("foo", Position::fake()).into(),
            &Default::default(),
            &[(
                "foo".into(),
                vec![types::RecordField::new(
                    "foo",
                    types::None::new(Position::fake())
                )]
            )]
            .into_iter()
            .collect()
        )
        .unwrap());
    }

    #[test]
    fn check_record_type_with_function_field() {
        assert!(!check(
            &types::Record::new("foo", Position::fake()).into(),
            &Default::default(),
            &[(
                "foo".into(),
                vec![types::RecordField::new(
                    "x",
                    types::Function::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Position::fake(),
                    )
                )]
            )]
            .into_iter()
            .collect()
        )
        .unwrap());
    }

    #[test]
    fn check_comparability_of_record_type_with_any_field() {
        assert!(!check(
            &types::Record::new("foo", Position::fake()).into(),
            &Default::default(),
            &[(
                "foo".into(),
                vec![types::RecordField::new(
                    "x",
                    types::Any::new(Position::fake())
                )]
            )]
            .into_iter()
            .collect()
        )
        .unwrap());
    }

    #[test]
    fn check_union_type() {
        assert!(check(
            &types::Union::new(
                types::Number::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake()
            )
            .into(),
            &Default::default(),
            &Default::default(),
        )
        .unwrap());
    }

    #[test]
    fn check_union_type_with_function() {
        assert!(!check(
            &types::Union::new(
                types::Function::new(vec![], types::None::new(Position::fake()), Position::fake(),),
                types::None::new(Position::fake()),
                Position::fake()
            )
            .into(),
            &Default::default(),
            &Default::default(),
        )
        .unwrap());
    }
}
