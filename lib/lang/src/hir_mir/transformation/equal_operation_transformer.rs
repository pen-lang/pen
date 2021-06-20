use super::super::error::CompileError;
use crate::{hir::*, position::Position, types::Type};

pub fn transform(operation: &EqualityOperation) -> Result<Expression, CompileError> {
    Ok(if operation.operator() == EqualityOperator::Equal {
        transform_equal_operation(
            operation
                .type_()
                .ok_or_else(|| CompileError::TypeNotInferred(operation.position().clone()))?,
            operation.lhs(),
            operation.rhs(),
            operation.position(),
        )?
    } else {
        operation.clone().into()
    })
}

fn transform_equal_operation(
    _type_: &Type,
    _lhs: &Expression,
    _rhs: &Expression,
    _position: &Position,
) -> Result<Expression, CompileError> {
    todo!()
}
