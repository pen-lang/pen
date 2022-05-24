use crate::{context::Context, error::CompileError, type_};

pub fn get_field(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    record: &fmm::build::TypedExpression,
    record_type: &mir::types::Record,
    field_index: usize,
) -> Result<fmm::build::TypedExpression, CompileError> {
    get_unboxed_field(
        builder,
        &if type_::is_record_boxed(record_type, context.types()) {
            load(context, builder, record, record_type)?
        } else {
            record.clone()
        },
        field_index,
    )
}

pub fn get_unboxed_field(
    builder: &fmm::build::InstructionBuilder,
    record: &fmm::build::TypedExpression,
    field_index: usize,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.deconstruct_record(record.clone(), field_index)?)
}

pub fn load(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    record: &fmm::build::TypedExpression,
    record_type: &mir::types::Record,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.load(fmm::build::bit_cast(
        fmm::types::Pointer::new(type_::compile_unboxed_record(record_type, context.types())),
        record.clone(),
    ))?)
}
