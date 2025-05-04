use crate::{CompileError, box_, context::Context, type_};

pub fn convert_to_foreign(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    value: impl Into<fmm::build::TypedExpression>,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let value = value.into();

    Ok(if type_::foreign::is_payload_boxed(context, type_)? {
        box_::box_(builder, value)?
    } else {
        value
    })
}

pub fn convert_from_foreign(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    value: impl Into<fmm::build::TypedExpression>,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let value = value.into();

    Ok(if type_::foreign::is_payload_boxed(context, type_)? {
        box_::unbox(context, builder, value, type_)?
    } else {
        value
    })
}
