use super::{super::error::CompileError, function, pointer, record};
use crate::{type_information, variant};
use fnv::FnvHashMap;

pub fn clone(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(match type_ {
        mir::types::Type::ByteString => pointer::clone(builder, expression)?,
        mir::types::Type::Function(_) => function::clone(builder, expression)?,
        mir::types::Type::Record(record) => builder.call(
            fmm::build::variable(
                record::utilities::get_clone_function_name(record.name()),
                record::utilities::compile_clone_function_type(record, types),
            ),
            vec![expression.clone()],
        )?,
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
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<(), CompileError> {
    match type_ {
        mir::types::Type::ByteString => pointer::drop(builder, expression, |_| Ok(()))?,
        mir::types::Type::Function(_) => function::drop(builder, expression)?,
        mir::types::Type::Record(record) => {
            builder.call(
                fmm::build::variable(
                    record::utilities::get_drop_function_name(record.name()),
                    record::utilities::compile_drop_function_type(record, types),
                ),
                vec![expression.clone()],
            )?;
        }
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
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<(), CompileError> {
    match type_ {
        mir::types::Type::ByteString => {
            pointer::synchronize(builder, expression, |_| Ok(()))?;
        }
        mir::types::Type::Function(_) => {
            function::synchronize(builder, expression)?;
        }
        mir::types::Type::Record(record) => {
            builder.call(
                fmm::build::variable(
                    record::utilities::get_synchronize_function_name(record.name()),
                    record::utilities::compile_synchronize_function_type(record, types),
                ),
                vec![expression.clone()],
            )?;
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
