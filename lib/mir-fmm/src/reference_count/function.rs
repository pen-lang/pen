use super::{super::error::CompileError, pointer};
use crate::closure;

pub fn clone(
    builder: &fmm::build::InstructionBuilder,
    closure_pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    pointer::clone(builder, closure_pointer)
}

pub fn drop(
    builder: &fmm::build::InstructionBuilder,
    closure_pointer: &fmm::build::TypedExpression,
) -> Result<(), CompileError> {
    pointer::drop(builder, closure_pointer, |builder| {
        builder.call(
            closure::metadata::load_drop_function(
                builder,
                builder.load(closure::get_metadata_pointer(closure_pointer.clone())?)?,
            )?,
            vec![fmm::build::bit_cast(
                fmm::types::Primitive::PointerInteger,
                closure_pointer.clone(),
            )
            .into()],
        )?;

        Ok(())
    })
}

pub fn synchronize(
    builder: &fmm::build::InstructionBuilder,
    closure_pointer: &fmm::build::TypedExpression,
) -> Result<(), CompileError> {
    pointer::synchronize(builder, closure_pointer, |builder| {
        builder.call(
            closure::metadata::load_synchronize_function(
                builder,
                builder.load(closure::get_metadata_pointer(closure_pointer.clone())?)?,
            )?,
            vec![fmm::build::bit_cast(
                fmm::types::Primitive::PointerInteger,
                closure_pointer.clone(),
            )
            .into()],
        )?;

        Ok(())
    })
}
