use super::AnalysisError;
use crate::types::Type;
use fnv::FnvHashMap;

pub fn format(
    type_: &Type,
    original_names: &FnvHashMap<String, String>,
) -> Result<String, AnalysisError> {
    let format = |type_| format(type_, original_names);

    Ok(match type_ {
        Type::Any(_) => "any".into(),
        Type::Boolean(_) => "boolean".into(),
        Type::Error(_) => "error".into(),
        Type::Function(function) => format!(
            "\\({}) {}",
            &function
                .arguments()
                .iter()
                .map(format)
                .collect::<Result<Vec<_>, _>>()?
                .join(", "),
            format(function.result())?,
        ),
        Type::List(list) => format!("[{}]", format(list.element())?),
        Type::Map(map) => format!("{{{}: {}}}", format(map.key())?, format(map.value())?),
        Type::None(_) => "none".into(),
        Type::Number(_) => "number".into(),
        Type::Record(record) => original_names
            .get(record.name())
            .ok_or_else(|| AnalysisError::RecordNotFound(record.clone()))?
            .clone(),
        Type::Reference(reference) => original_names
            .get(reference.name())
            .ok_or_else(|| AnalysisError::TypeNotFound(reference.clone()))?
            .clone(),
        Type::String(_) => "string".into(),
        Type::Union(union) => format!("{} | {}", format(union.lhs())?, format(union.rhs())?),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use position::{test::PositionFake, Position};

    #[test]
    fn format_function() {
        assert_eq!(
            format(
                &Function::new(vec![], None::new(Position::fake()), Position::fake()).into(),
                &Default::default()
            ),
            Ok("\\() none".into())
        );
    }

    #[test]
    fn format_function_with_argument() {
        assert_eq!(
            format(
                &Function::new(
                    vec![None::new(Position::fake()).into()],
                    None::new(Position::fake()),
                    Position::fake()
                )
                .into(),
                &Default::default()
            ),
            Ok("\\(none) none".into())
        );
    }

    #[test]
    fn format_function_with_arguments() {
        assert_eq!(
            format(
                &Function::new(
                    vec![
                        Number::new(Position::fake()).into(),
                        None::new(Position::fake()).into()
                    ],
                    None::new(Position::fake()),
                    Position::fake()
                )
                .into(),
                &Default::default()
            ),
            Ok("\\(number, none) none".into())
        );
    }

    #[test]
    fn format_list() {
        assert_eq!(
            format(
                &List::new(None::new(Position::fake()), Position::fake()).into(),
                &Default::default()
            ),
            Ok("[none]".into())
        );
    }

    #[test]
    fn format_union() {
        assert_eq!(
            format(
                &Union::new(
                    Number::new(Position::fake()),
                    None::new(Position::fake()),
                    Position::fake()
                )
                .into(),
                &Default::default()
            ),
            Ok("number | none".into())
        );
    }

    #[test]
    fn format_unions() {
        assert_eq!(
            format(
                &Union::new(
                    None::new(Position::fake()),
                    Union::new(
                        Boolean::new(Position::fake()),
                        Number::new(Position::fake()),
                        Position::fake()
                    ),
                    Position::fake()
                )
                .into(),
                &Default::default()
            ),
            Ok("none | boolean | number".into())
        );
    }

    #[test]
    fn format_record() {
        assert_eq!(
            format(
                &Record::new("foo", Position::fake()).into(),
                &[("foo".into(), "bar".into())].into_iter().collect(),
            ),
            Ok("bar".into())
        );
    }

    #[test]
    fn format_reference() {
        assert_eq!(
            format(
                &Reference::new("foo", Position::fake()).into(),
                &[("foo".into(), "bar".into())].into_iter().collect(),
            ),
            Ok("bar".into())
        );
    }

    #[test]
    fn format_map() {
        assert_eq!(
            format(
                &Map::new(
                    Number::new(Position::fake()),
                    None::new(Position::fake()),
                    Position::fake()
                )
                .into(),
                &Default::default(),
            ),
            Ok("{number: none}".into())
        );
    }
}
