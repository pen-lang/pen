mod count;
mod expression;
mod function;
pub mod heap;
pub mod pointer;
pub mod record;
pub mod variant;

use crate::CompileError;
pub use expression::*;

pub fn compile_static(
    expression: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::record(vec![count::compile_static()?, expression.into()]).into())
}

pub fn compile_type_with_reference_count(type_: impl Into<fmm::types::Type>) -> fmm::types::Record {
    fmm::types::Record::new(vec![count::compile_type().into(), type_.into()])
}
