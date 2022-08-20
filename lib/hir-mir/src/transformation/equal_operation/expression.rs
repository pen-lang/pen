use super::operation;
use crate::context::CompileContext;
use crate::error::CompileError;
use hir::{analysis::AnalysisError, ir::*};

pub fn transform(
    context: &CompileContext,
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
