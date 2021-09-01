use hir::ir::*;

pub fn transform(operation: &EqualityOperation) -> Expression {
    if operation.operator() == EqualityOperator::NotEqual {
        let position = operation.position();

        If::new(
            EqualityOperation::new(
                operation.type_().cloned(),
                EqualityOperator::Equal,
                operation.lhs().clone(),
                operation.rhs().clone(),
                position.clone(),
            ),
            Boolean::new(false, position.clone()),
            Boolean::new(true, position.clone()),
            position.clone(),
        )
        .into()
    } else {
        operation.clone().into()
    }
}
