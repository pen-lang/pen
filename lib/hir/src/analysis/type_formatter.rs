use crate::types::Type;

pub fn format(type_: &Type) -> String {
    match type_ {
        Type::Any(_) => "any".into(),
        Type::Boolean(_) => "boolean".into(),
        Type::Error(_) => "error".into(),
        Type::Function(function) => format!(
            "\\({}) {}",
            &function
                .arguments()
                .iter()
                .map(format)
                .collect::<Vec<_>>()
                .join(", "),
            format(function.result()),
        ),
        Type::List(list) => format!("[{}]", format(list.element())),
        Type::Map(map) => format!("{{{}: {}}}", format(map.key()), format(map.value())),
        Type::None(_) => "none".into(),
        Type::Number(_) => "number".into(),
        Type::Record(record) => record.original_name().into(),
        Type::Reference(reference) => reference.name().into(),
        Type::String(_) => "string".into(),
        Type::Union(union) => format!("{} | {}", format(union.lhs()), format(union.rhs())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{self, *};
    use position::{Position, test::PositionFake};

    #[test]
    fn format_function() {
        assert_eq!(
            format(&Function::new(vec![], None::new(Position::fake()), Position::fake()).into(),),
            "\\() none"
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
            ),
            "\\(none) none"
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
            ),
            "\\(number, none) none"
        );
    }

    #[test]
    fn format_list() {
        assert_eq!(
            format(&List::new(None::new(Position::fake()), Position::fake()).into(),),
            "[none]"
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
            ),
            "number | none"
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
            ),
            "none | boolean | number"
        );
    }

    #[test]
    fn format_record() {
        assert_eq!(
            format(&types::Record::new("foo", "bar", Position::fake()).into(),),
            "bar"
        );
    }

    #[test]
    fn format_reference() {
        assert_eq!(
            format(&Reference::new("foo", Position::fake()).into(),),
            "foo"
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
            ),
            "{number: none}"
        );
    }
}
