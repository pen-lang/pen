use hir::ir::*;

pub fn transform(operation: &BooleanOperation) -> Expression {
    let position = operation.position();

    match operation.operator() {
        BooleanOperator::And => If::new(
            operation.lhs().clone(),
            operation.rhs().clone(),
            Boolean::new(false, position.clone()),
            position.clone(),
        )
        .into(),
        BooleanOperator::Or => If::new(
            operation.lhs().clone(),
            Boolean::new(true, position.clone()),
            operation.rhs().clone(),
            position.clone(),
        )
        .into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;
    use pretty_assertions::assert_eq;

    #[test]
    fn transform_and_operation() {
        assert_eq!(
            transform(&BooleanOperation::new(
                BooleanOperator::And,
                Boolean::new(true, test::position()),
                Boolean::new(true, test::position()),
                test::position(),
            )),
            If::new(
                Boolean::new(true, test::position()),
                Boolean::new(true, test::position()),
                Boolean::new(false, test::position()),
                test::position(),
            )
            .into(),
        );
    }

    #[test]
    fn transform_or_operation() {
        assert_eq!(
            transform(&BooleanOperation::new(
                BooleanOperator::Or,
                Boolean::new(false, test::position()),
                Boolean::new(false, test::position()),
                test::position(),
            )),
            If::new(
                Boolean::new(false, test::position()),
                Boolean::new(true, test::position()),
                Boolean::new(false, test::position()),
                test::position(),
            )
            .into(),
        );
    }
}
