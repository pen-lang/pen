use crate::BinaryOperator;

pub fn is_name_public(name: &str) -> bool {
    name.chars()
        .next()
        .map(|character| character.is_ascii_uppercase())
        .unwrap_or_default()
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
