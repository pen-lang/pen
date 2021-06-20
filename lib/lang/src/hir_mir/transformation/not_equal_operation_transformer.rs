use crate::hir::*;

pub fn transform(operation: &EqualityOperation) -> Expression {
    if operation.operator() == EqualityOperator::NotEqual {
        todo!()
        // If::new(
        //     EqualityOperation::new(
        //         operation.type_().clone(),
        //         EqualityOperator::Equal,
        //         operation.lhs().clone(),
        //         operation.rhs().clone(),
        //         position.clone(),
        //     ),
        //     Boolean::new(false, position.clone()),
        //     Boolean::new(true, position.clone()),
        //     position.clone(),
        // )
        // .into()
    } else {
        operation.clone().into()
    }
}
