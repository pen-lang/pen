use crate::hir::*;

pub fn transform(_operation: &BooleanOperation) -> Expression {
    todo!()
    // let position = operation.position();
    //
    // match operation.operator() {
    //     BooleanOperator::And => If::new(
    //         operation.lhs().clone(),
    //         operation.rhs().clone(),
    //         Boolean::new(false, position.clone()),
    //         Some(types::Boolean::new(Position::dummy())),
    //         position.clone(),
    //     )
    //     .into(),
    //     BooleanOperator::Or => If::new(
    //         operation.lhs().clone(),
    //         Boolean::new(true, position.clone()),
    //         operation.rhs().clone(),
    //         Some(types::Boolean::new(Position::dummy())),
    //         position.clone(),
    //     )
    //     .into(),
    // }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::position::Position;
    // use pretty_assertions::assert_eq;

    // #[test]
    // fn transform_and_operation() {
    //     assert_eq!(
    //         transform(&BooleanOperation::new(
    //             BooleanOperator::And,
    //             Boolean::new(true, Position::dummy()),
    //             Boolean::new(true, Position::dummy()),
    //             Position::dummy(),
    //         )),
    //         If::new(
    //             Boolean::new(true, Position::dummy()),
    //             Boolean::new(true, Position::dummy()),
    //             Boolean::new(false, Position::dummy()),
    //             Position::dummy(),
    //         )
    //         .into(),
    //     );
    // }

    // #[test]
    // fn transform_or_operation() {
    //     assert_eq!(
    //         transform(&BooleanOperation::new(
    //             BooleanOperator::Or,
    //             Boolean::new(false, Position::dummy()),
    //             Boolean::new(false, Position::dummy()),
    //             Position::dummy(),
    //         )),
    //         If::new(
    //             Boolean::new(false, Position::dummy()),
    //             Boolean::new(true, Position::dummy()),
    //             Boolean::new(false, Position::dummy()),
    //             Position::dummy(),
    //         )
    //         .into(),
    //     );
    // }
}
