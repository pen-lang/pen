use crate::{CompileError, concrete_type, context::Context, type_};
use hir::types::Type;

pub fn compile_any(
    context: &Context,
    value: impl Into<mir::ir::Expression>,
    type_: &Type,
) -> Result<mir::ir::Expression, CompileError> {
    Ok(
        if type_::compile(context, type_)? == mir::types::Type::Variant {
            value.into()
        } else {
            mir::ir::Variant::new(
                type_::compile_concrete(context, type_)?,
                concrete_type::compile(context, value, type_)?,
            )
            .into()
        },
    )
}

pub fn compile_unboxed_concrete(
    context: &Context,
    expression: impl Into<mir::ir::Expression>,
    type_: &Type,
) -> Result<mir::ir::Expression, CompileError> {
    Ok(mir::ir::RecordField::new(
        type_::compile_concrete(context, type_)?
            .into_record()
            .unwrap(),
        0,
        expression.into(),
    )
    .into())
}
