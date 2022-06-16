use super::{super::error::CompileError, function, pointer, record};
use crate::{
    type_information::{
        TYPE_INFORMATION_CLONE_FUNCTION_FIELD_INDEX, TYPE_INFORMATION_DROP_FUNCTION_FIELD_INDEX,
        TYPE_INFORMATION_SYNCHRONIZE_FUNCTION_FIELD_INDEX,
    },
    variant,
};
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

            fmm::build::record(vec![
                tag.clone(),
                builder.call(
                    builder.deconstruct_record(
                        builder.load(tag)?,
                        TYPE_INFORMATION_CLONE_FUNCTION_FIELD_INDEX,
                    )?,
                    vec![variant::get_payload(builder, expression)?],
                )?,
            ])
            .into()
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
                builder.deconstruct_record(
                    builder.load(variant::get_tag(builder, expression)?)?,
                    TYPE_INFORMATION_DROP_FUNCTION_FIELD_INDEX,
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
                builder.deconstruct_record(
                    builder.load(variant::get_tag(builder, expression)?)?,
                    TYPE_INFORMATION_SYNCHRONIZE_FUNCTION_FIELD_INDEX,
                )?,
                vec![variant::get_payload(builder, expression)?],
            )?;
        }
        mir::types::Type::Boolean | mir::types::Type::None | mir::types::Type::Number => {}
    }

    Ok(())
}
