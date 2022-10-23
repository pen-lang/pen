use super::{super::error::CompileError, function, record, string, variant};
use crate::context::Context;

pub fn clone(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(match type_ {
        mir::types::Type::ByteString => string::clone(builder, expression)?,
        mir::types::Type::Function(_) => function::clone(builder, expression)?,
        mir::types::Type::Record(record) => record::clone(context, builder, expression, record)?,
        mir::types::Type::Variant => variant::clone(builder, expression)?,
        mir::types::Type::Boolean | mir::types::Type::None | mir::types::Type::Number => {
            expression.clone()
        }
    })
}

pub fn drop(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
) -> Result<(), CompileError> {
    match type_ {
        mir::types::Type::ByteString => string::drop(builder, expression)?,
        mir::types::Type::Function(_) => function::drop(context, builder, expression)?,
        mir::types::Type::Record(record) => record::drop(context, builder, expression, record)?,
        mir::types::Type::Variant => variant::drop(builder, expression)?,
        mir::types::Type::Boolean | mir::types::Type::None | mir::types::Type::Number => {}
    }

    Ok(())
}

pub fn synchronize(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
) -> Result<(), CompileError> {
    match type_ {
        mir::types::Type::ByteString => string::synchronize(builder, expression)?,
        mir::types::Type::Function(_) => function::synchronize(context, builder, expression)?,
        mir::types::Type::Record(record) => {
            record::synchronize(context, builder, expression, record)?
        }
        mir::types::Type::Variant => variant::synchronize(builder, expression)?,
        mir::types::Type::Boolean | mir::types::Type::None | mir::types::Type::Number => {}
    }

    Ok(())
}
