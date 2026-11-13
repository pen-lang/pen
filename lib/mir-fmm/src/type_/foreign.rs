use crate::{CompileError, context::Context, type_};

pub fn compile(
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

pub fn compile_function(
    context: &Context,
    function: &mir::types::Function,
    calling_convention: mir::ir::CallingConvention,
) -> Result<fmm::types::Function, CompileError> {
    Ok(fmm::types::Function::new(
        function
            .arguments()
            .iter()
            .map(|type_| compile(context, type_))
            .collect::<Result<_, _>>()?,
        compile(context, function.result())?,
        type_::compile_calling_convention(calling_convention),
    ))
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
        mir::types::Type::Boolean
        | mir::types::Type::ByteString
        | mir::types::Type::Function(_)
        | mir::types::Type::None
        | mir::types::Type::Number
        | mir::types::Type::Variant => false,
    })
}

fn is_record_boxed(context: &Context, record: &mir::types::Record) -> bool {
    type_::is_record_boxed(context, record)
}
