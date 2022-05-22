use crate::{type_, CompileError};
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
                box_(builder, value)
            } else {
                return Err(CompileError::UnboxedForeignRecord);
            }
        }
        mir::types::Type::Variant => box_(builder, value),
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
                unbox(builder, value)?
            } else {
                return Err(CompileError::UnboxedForeignRecord);
            }
        }
        mir::types::Type::Variant => unbox(builder, value)?,
        mir::types::Type::Boolean
        | mir::types::Type::ByteString
        | mir::types::Type::Function(_)
        | mir::types::Type::None
        | mir::types::Type::Number => value,
    })
}

fn box_(
    builder: &fmm::build::InstructionBuilder,
    unboxed: fmm::build::TypedExpression,
) -> fmm::build::TypedExpression {
    let pointer = builder.allocate_heap(fmm::build::size_of(unboxed.type_().clone()));

    builder.store(unboxed, pointer.clone());

    pointer
}

fn unbox(
    builder: &fmm::build::InstructionBuilder,
    pointer: fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let unboxed = builder.load(pointer.clone())?;

    builder.free_heap(pointer);

    Ok(unboxed)
}
