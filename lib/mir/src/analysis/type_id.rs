use crate::types::Type;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub fn calculate(type_: &Type) -> String {
    let mut hasher = DefaultHasher::new();

    calculate_string(type_).hash(&mut hasher);

    format!("{:x}", hasher.finish())
}

fn calculate_string(type_: &Type) -> String {
    match type_ {
        Type::Boolean => "boolean".into(),
        Type::ByteString => "string".into(),
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
        Type::None => "none".into(),
        Type::Number => "number".into(),
        Type::Record(record) => record.name().into(),
        Type::Variant => "variant".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types;

    #[test]
    fn calculate_none() {
        assert_eq!(calculate_string(&Type::None,), "none");
    }

    #[test]
    fn calculate_function() {
        assert_eq!(
            calculate_string(
                &types::Function::new(vec![Type::Boolean, Type::None], Type::Number).into()
            ),
            "(\\(boolean,none)number)"
        );
    }

    #[test]
    fn calculate_record() {
        assert_eq!(calculate_string(&types::Record::new("foo").into()), "foo");
    }
}
