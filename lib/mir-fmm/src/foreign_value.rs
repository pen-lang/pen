use crate::{reference_count, type_, CompileError};
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
                box_(builder, value)?
            } else {
                return Err(CompileError::UnboxedForeignRecord);
            }
        }
        mir::types::Type::Variant => box_(builder, value)?,
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
                unbox(builder, value, type_, types)?
            } else {
                return Err(CompileError::UnboxedForeignRecord);
            }
        }
        mir::types::Type::Variant => unbox(builder, value, type_, types)?,
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
) -> Result<fmm::build::TypedExpression, CompileError> {
    let pointer = reference_count::heap::allocate(builder, unboxed.type_().clone())?;

    builder.store(unboxed, pointer.clone());

    Ok(pointer)
}

fn unbox(
    builder: &fmm::build::InstructionBuilder,
    pointer: fmm::build::TypedExpression,
    type_: &mir::types::Type,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let load = |builder: &fmm::build::InstructionBuilder| {
        builder.load(fmm::build::bit_cast(
            fmm::types::Pointer::new(type_::compile(type_, types)),
            pointer.clone(),
        ))
    };
    let unboxed = reference_count::clone(builder, &load(builder)?, type_, types)?;

    reference_count::pointer::drop(builder, &pointer, |builder| {
        reference_count::drop(builder, &load(builder)?, type_, types)?;

        Ok(())
    })?;

    Ok(unboxed)
}
