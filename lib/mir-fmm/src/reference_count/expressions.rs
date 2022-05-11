use super::{super::error::CompileError, functions, pointers, record_utilities};
use crate::{
    type_information::{
        TYPE_INFORMATION_CLONE_FUNCTION_FIELD_INDEX, TYPE_INFORMATION_DROP_FUNCTION_FIELD_INDEX,
    },
    types,
    variants::{VARIANT_PAYLOAD_FIELD_INDEX, VARIANT_TAG_FIELD_INDEX},
};
use fnv::FnvHashMap;

pub fn clone_expression(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
    types: &FnvHashMap<String, mir::types::RecordBody>,
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
    types: &FnvHashMap<String, mir::types::RecordBody>,
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

pub fn drop_or_reuse_expression(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let drop = || -> Result<_, CompileError> {
        drop_expression(builder, expression, type_, types)?;

        Ok(fmm::build::TypedExpression::from(fmm::ir::Undefined::new(
            fmm::types::GENERIC_POINTER_TYPE.clone(),
        )))
    };

    Ok(match type_ {
        mir::types::Type::Record(record) => {
            if types::is_record_boxed(record, types) {
                fmm::build::bit_cast(
                    fmm::types::GENERIC_POINTER_TYPE.clone(),
                    builder.call(
                        fmm::build::variable(
                            record_utilities::get_record_drop_or_reuse_function_name(record.name()),
                            record_utilities::compile_record_drop_or_reuse_function_type(
                                record, types,
                            ),
                        ),
                        vec![expression.clone()],
                    )?,
                )
                .into()
            } else {
                drop()?
            }
        }
        mir::types::Type::ByteString
        | mir::types::Type::Boolean
        | mir::types::Type::Function(_)
        | mir::types::Type::None
        | mir::types::Type::Number
        | mir::types::Type::Variant => drop()?,
    })
}
