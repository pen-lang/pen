use crate::{types, CompileError};
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
            if types::is_record_boxed(record_type, types) {
                value.clone()
            } else {
                todo!()
            }
        }
        mir::types::Type::Variant => {
            let unboxed = fmm::build::record(vec![value]);
            let pointer = builder.allocate_heap(fmm::build::size_of(unboxed.type_().clone()));

            builder.store(unboxed, pointer.clone());

            pointer
        }
        mir::types::Type::Boolean
        | mir::types::Type::ByteString
        | mir::types::Type::Function(_)
        | mir::types::Type::None
        | mir::types::Type::Number => value.clone(),
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
            if types::is_record_boxed(record_type, types) {
                todo!()
            } else {
                todo!()
            }
        }
        mir::types::Type::Variant => {
            let unboxed = builder.load(value.clone())?;

            builder.free_heap(value);

            unboxed
        }
        mir::types::Type::Boolean
        | mir::types::Type::ByteString
        | mir::types::Type::Function(_)
        | mir::types::Type::None
        | mir::types::Type::Number => value.clone(),
    })
}
