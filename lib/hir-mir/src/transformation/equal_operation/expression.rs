use super::operation;
use crate::{context::Context, error::CompileError};
use hir::{analysis::AnalysisError, ir::*};

pub fn transform(
    context: &Context,
    operation: &EqualityOperation,
) -> Result<Expression, CompileError> {
    Ok(if operation.operator() == EqualityOperator::Equal {
        operation::transform(
            context,
            operation
                .type_()
                .ok_or_else(|| AnalysisError::TypeNotInferred(operation.position().clone()))?,
            operation.lhs(),
            operation.rhs(),
            operation.position(),
        )?
    } else {
        operation.clone().into()
    })
}
