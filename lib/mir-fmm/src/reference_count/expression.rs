use super::{super::error::CompileError, function, pointer, record_utilities};
use crate::{
    type_,
    type_information::{
        TYPE_INFORMATION_CLONE_FUNCTION_FIELD_INDEX, TYPE_INFORMATION_DROP_FUNCTION_FIELD_INDEX,
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
                record_utilities::get_record_clone_function_name(record.name()),
                record_utilities::compile_record_clone_function_type(record, types),
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
                    record_utilities::get_record_drop_function_name(record.name()),
                    record_utilities::compile_record_drop_function_type(record, types),
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

pub fn drop_or_reuse(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let pointer_type = fmm::types::GENERIC_POINTER_TYPE.clone();
    let drop = || -> Result<_, CompileError> {
        drop(builder, expression, type_, types)?;

        Ok(fmm::ir::Undefined::new(pointer_type.clone()).into())
    };

    Ok(match type_ {
        mir::types::Type::Record(record_type) => {
            if type_::is_record_boxed(record_type, types) {
                fmm::build::bit_cast(
                    pointer_type,
                    builder.call(
                        fmm::build::variable(
                            record_utilities::get_record_drop_or_reuse_function_name(
                                record_type.name(),
                            ),
                            record_utilities::compile_record_drop_or_reuse_function_type(
                                record_type,
                                types,
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
