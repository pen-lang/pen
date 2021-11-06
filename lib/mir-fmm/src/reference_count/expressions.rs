use super::{super::error::CompileError, functions, pointers, record_utilities};
use crate::{
    type_information::{
        TYPE_INFORMATION_CLONE_FUNCTION_FIELD_INDEX, TYPE_INFORMATION_DROP_FUNCTION_FIELD_INDEX,
    },
    variants::{VARIANT_PAYLOAD_FIELD_INDEX, VARIANT_TAG_FIELD_INDEX},
};
use std::collections::BTreeMap;

pub fn clone_expression(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
    types: &BTreeMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(match type_ {
        mir::types::Type::ByteString => pointers::clone_pointer(builder, expression)?,
        mir::types::Type::Function(_) => functions::clone_function(builder, expression)?,
        mir::types::Type::Record(record) => builder.call(
            fmm::build::variable(
                record_utilities::get_record_clone_function_name(record.name()),
                record_utilities::compile_record_clone_function_type(record, types),
            ),
            vec![expression.clone()],
        )?,
        mir::types::Type::Variant => {
            let tag = builder.deconstruct_record(expression.clone(), VARIANT_TAG_FIELD_INDEX)?;

            fmm::build::record(vec![
                tag.clone(),
                builder.call(
                    builder.deconstruct_record(
                        builder.load(tag)?,
                        TYPE_INFORMATION_CLONE_FUNCTION_FIELD_INDEX,
                    )?,
                    vec![builder
                        .deconstruct_record(expression.clone(), VARIANT_PAYLOAD_FIELD_INDEX)?],
                )?,
            ])
            .into()
        }
        mir::types::Type::Boolean | mir::types::Type::None | mir::types::Type::Number => {
            expression.clone()
        }
    })
}

pub fn drop_expression(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
    types: &BTreeMap<String, mir::types::RecordBody>,
) -> Result<(), CompileError> {
    match type_ {
        mir::types::Type::ByteString => pointers::drop_pointer(builder, expression, |_| Ok(()))?,
        mir::types::Type::Function(_) => functions::drop_function(builder, expression)?,
        mir::types::Type::Record(record) => {
            builder.call(
                fmm::build::variable(
                    record_utilities::get_record_drop_function_name(record.name()),
                    record_utilities::compile_record_drop_function_type(record, types),
                ),
                vec![expression.clone()],
            )?;
        }
        mir::types::Type::Variant => {
            builder.call(
                builder.deconstruct_record(
                    builder.load(
                        builder.deconstruct_record(expression.clone(), VARIANT_TAG_FIELD_INDEX)?,
                    )?,
                    TYPE_INFORMATION_DROP_FUNCTION_FIELD_INDEX,
                )?,
                vec![builder.deconstruct_record(expression.clone(), VARIANT_PAYLOAD_FIELD_INDEX)?],
            )?;
        }
        mir::types::Type::Boolean | mir::types::Type::None | mir::types::Type::Number => {}
    }

    Ok(())
}
