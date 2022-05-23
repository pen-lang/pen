use crate::{box_, type_, CompileError};
use fnv::FnvHashMap;

pub fn convert_to_foreign(
    builder: &fmm::build::InstructionBuilder,
    value: impl Into<fmm::build::TypedExpression>,
    type_: &mir::types::Type,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let value = value.into();

    Ok(match type_ {
        mir::types::Type::Record(record_type) => {
            if type_::is_record_boxed(record_type, types)
                == type_::is_foreign_record_boxed(record_type, types)
            {
                value
            } else if !type_::is_record_boxed(record_type, types)
                && type_::is_foreign_record_boxed(record_type, types)
            {
                box_::box_(builder, value)?
            } else {
                return Err(CompileError::UnboxedForeignRecord);
            }
        }
        mir::types::Type::Variant => box_::box_(builder, value)?,
        mir::types::Type::Boolean
        | mir::types::Type::ByteString
        | mir::types::Type::Function(_)
        | mir::types::Type::None
        | mir::types::Type::Number => value,
    })
}

pub fn convert_from_foreign(
    builder: &fmm::build::InstructionBuilder,
    value: impl Into<fmm::build::TypedExpression>,
    type_: &mir::types::Type,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let value = value.into();

    Ok(match type_ {
        mir::types::Type::Record(record_type) => {
            if type_::is_record_boxed(record_type, types)
                == type_::is_foreign_record_boxed(record_type, types)
            {
                value
            } else if !type_::is_record_boxed(record_type, types)
                && type_::is_foreign_record_boxed(record_type, types)
            {
                box_::unbox(builder, value, type_, types)?
            } else {
                return Err(CompileError::UnboxedForeignRecord);
            }
        }
        mir::types::Type::Variant => box_::unbox(builder, value, type_, types)?,
        mir::types::Type::Boolean
        | mir::types::Type::ByteString
        | mir::types::Type::Function(_)
        | mir::types::Type::None
        | mir::types::Type::Number => value,
    })
}
