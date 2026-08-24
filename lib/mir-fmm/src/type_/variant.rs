use crate::{CompileError, context::Context, type_};

pub fn compile_payload(
    context: &Context,
    type_: &mir::types::Type,
) -> Result<fmm::types::Type, CompileError> {
    let fmm_type = type_::compile(context, type_);

    Ok(if is_payload_boxed(context, type_)? {
        fmm::types::Pointer::new(fmm_type).into()
    } else {
        fmm_type
    })
}

pub fn is_payload_boxed(context: &Context, type_: &mir::types::Type) -> Result<bool, CompileError> {
    Ok(match type_ {
        mir::types::Type::Record(record_type) => {
            if type_::is_record_boxed(context, record_type)
                && !is_record_boxed(context, record_type)
            {
                return Err(CompileError::UnboxedRecord);
            }

            !type_::is_record_boxed(context, record_type) && is_record_boxed(context, record_type)
        }
        mir::types::Type::Variant => return Err(CompileError::NestedVariant),
        mir::types::Type::Boolean
        | mir::types::Type::ByteString
        | mir::types::Type::Function(_)
        | mir::types::Type::None
        | mir::types::Type::Number => false,
    })
}

// Box large records to stuff them into one word.
fn is_record_boxed(context: &Context, record: &mir::types::Record) -> bool {
    // TODO Unbox small records.
    type_::is_record_boxed(context, record)
}
