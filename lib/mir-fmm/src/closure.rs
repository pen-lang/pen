mod drop;
pub mod metadata;
mod sync;

use super::{reference_count, CompileError};

pub fn get_entry_function_pointer(
    closure_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    get_field(closure_pointer, 0)
}

pub fn get_metadata_pointer(
    closure_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    get_field(closure_pointer, 1)
}

pub fn get_payload_pointer(
    closure_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    get_field(closure_pointer, 2)
}

fn get_field(
    closure_pointer: impl Into<fmm::build::TypedExpression>,
    index: usize,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::record_address(
        reference_count::pointer::untag(&closure_pointer.into())?,
        index,
    )?
    .into())
}

pub fn compile_content(
    entry_function: impl Into<fmm::build::TypedExpression>,
    metadata: impl Into<fmm::build::TypedExpression>,
    payload: impl Into<fmm::build::TypedExpression>,
) -> fmm::build::TypedExpression {
    fmm::build::record(vec![entry_function.into(), metadata.into(), payload.into()]).into()
}
