use crate::{BinaryOperator, ModulePath, RecordDefinition};

pub fn is_name_public(name: &str) -> bool {
    name.chars()
        .next()
        .map(|character| character.is_ascii_uppercase())
        .unwrap_or_default()
}

pub fn is_record_open(definition: &RecordDefinition) -> bool {
    definition
        .fields()
        .iter()
        .all(|field| is_name_public(field.name()))
}

pub fn is_module_path_public(path: &ModulePath) -> bool {
    match path {
        ModulePath::External(path) => path.components(),
        ModulePath::Internal(path) => path.components(),
    }
    .iter()
    .all(|component| is_name_public(component))
}

pub fn operator_priority(operator: BinaryOperator) -> usize {
    match operator {
        BinaryOperator::Or => 1,
        BinaryOperator::And => 2,
        BinaryOperator::Equal
        | BinaryOperator::NotEqual
        | BinaryOperator::LessThan
        | BinaryOperator::LessThanOrEqual
        | BinaryOperator::GreaterThan
        | BinaryOperator::GreaterThanOrEqual => 3,
        BinaryOperator::Add | BinaryOperator::Subtract => 4,
        BinaryOperator::Multiply | BinaryOperator::Divide => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_public_name() {
        assert!(is_name_public("Foo"));
        assert!(!is_name_public("foo"));
    }
}
