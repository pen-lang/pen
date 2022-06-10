mod drop;
pub mod metadata;

use super::{reference_count, CompileError};

pub fn get_entry_function_pointer(
    closure_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(
        fmm::build::record_address(reference_count::pointer::untag(&closure_pointer.into())?, 0)?
            .into(),
    )
}

pub fn get_metadata_pointer(
    closure_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(
        fmm::build::record_address(reference_count::pointer::untag(&closure_pointer.into())?, 1)?
            .into(),
    )
}

pub fn load_metadata(
    builder: &fmm::build::InstructionBuilder,
    closure_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.load(get_metadata_pointer(closure_pointer)?)?)
}

pub fn store_metadata(
    builder: &fmm::build::InstructionBuilder,
    closure_pointer: impl Into<fmm::build::TypedExpression>,
    metadata: impl Into<fmm::build::TypedExpression>,
) -> Result<(), CompileError> {
    builder.store(metadata, get_metadata_pointer(closure_pointer)?);

    Ok(())
}

pub fn get_payload_pointer(
    closure_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(
        fmm::build::record_address(reference_count::pointer::untag(&closure_pointer.into())?, 2)?
            .into(),
    )
}

pub fn compile_content(
    entry_function: impl Into<fmm::build::TypedExpression>,
    metadata: impl Into<fmm::build::TypedExpression>,
    payload: impl Into<fmm::build::TypedExpression>,
) -> fmm::build::TypedExpression {
    fmm::build::record(vec![entry_function.into(), metadata.into(), payload.into()]).into()
}
