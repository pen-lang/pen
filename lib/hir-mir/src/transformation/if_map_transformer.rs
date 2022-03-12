use super::super::error::CompileError;
use crate::context::CompileContext;
use hir::ir::*;

pub fn transform(_context: &CompileContext, _if_: &IfMap) -> Result<Expression, CompileError> {
    todo!()
}
