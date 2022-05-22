use super::{super::error::CompileError, pointers};
use crate::closure;

pub fn clone_function(
    builder: &fmm::build::InstructionBuilder,
    closure_pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    pointers::clone_pointer(builder, closure_pointer)
}

pub fn drop_function(
    builder: &fmm::build::InstructionBuilder,
    closure_pointer: &fmm::build::TypedExpression,
) -> Result<(), CompileError> {
    pointers::drop_pointer(builder, closure_pointer, |builder| {
        builder.call(
            closure::compile_load_drop_function(builder, closure_pointer.clone())?,
            vec![fmm::build::bit_cast(
                fmm::types::Primitive::PointerInteger,
                closure_pointer.clone(),
            )
            .into()],
        )?;

        Ok(())
    })?;

    Ok(())
}
