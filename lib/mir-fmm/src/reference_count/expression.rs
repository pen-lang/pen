use super::{super::error::CompileError, function, pointer, record};
use crate::{context::Context, type_information, variant};

pub fn clone(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(match type_ {
        mir::types::Type::ByteString => pointer::clone(builder, expression)?,
        mir::types::Type::Function(_) => function::clone(builder, expression)?,
        mir::types::Type::Record(record) => record::clone(context, builder, expression, record)?,
        mir::types::Type::Variant => {
            let tag = variant::get_tag(builder, expression)?;

            builder.if_::<CompileError>(
                fmm::build::comparison_operation(
                    fmm::ir::ComparisonOperator::Equal,
                    fmm::build::bit_cast(fmm::types::Primitive::PointerInteger, tag.clone()),
                    fmm::ir::Undefined::new(fmm::types::Primitive::PointerInteger),
                )?,
                |builder| Ok(builder.branch(expression.clone())),
                |builder| {
                    Ok(builder.branch(fmm::build::record(vec![
                        tag.clone(),
                        builder.call(
                            type_information::get_clone_function(&builder, tag.clone())?,
                            vec![variant::get_payload(&builder, expression)?],
                        )?,
                    ])))
                },
            )?
        }
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
        mir::types::Type::ByteString => pointer::drop(builder, expression, |_| Ok(()))?,
        mir::types::Type::Function(_) => function::drop(context, builder, expression)?,
        mir::types::Type::Record(record) => record::drop(context, builder, expression, record)?,
        mir::types::Type::Variant => {
            builder.call(
                type_information::get_drop_function(
                    builder,
                    variant::get_tag(builder, expression)?,
                )?,
                vec![variant::get_payload(builder, expression)?],
            )?;
        }
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
        mir::types::Type::ByteString => pointer::synchronize(builder, expression, |_| Ok(()))?,
        mir::types::Type::Function(_) => function::synchronize(context, builder, expression)?,
        mir::types::Type::Record(record) => {
            record::synchronize(context, builder, expression, record)?
        }
        mir::types::Type::Variant => {
            builder.call(
                type_information::get_synchronize_function(
                    builder,
                    variant::get_tag(builder, expression)?,
                )?,
                vec![variant::get_payload(builder, expression)?],
            )?;
        }
        mir::types::Type::Boolean | mir::types::Type::None | mir::types::Type::Number => {}
    }

    Ok(())
}
