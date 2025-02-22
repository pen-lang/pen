use crate::{CompileError, context::Context, reference_count, type_};

pub fn box_(
    builder: &fmm::build::InstructionBuilder,
    unboxed: fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let pointer = reference_count::heap::allocate(builder, unboxed.type_().clone())?;

    builder.store(unboxed, pointer.clone());

    Ok(pointer)
}

pub fn unbox(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    pointer: fmm::build::TypedExpression,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let loaded = builder.load(fmm::build::bit_cast(
        fmm::types::Pointer::new(type_::compile(context, type_)),
        pointer.clone(),
    ))?;
    let unboxed = reference_count::clone(context, builder, &loaded, type_)?;

    reference_count::pointer::drop(builder, &pointer, |builder| {
        reference_count::drop(context, builder, &loaded, type_)
    })?;

    Ok(unboxed)
}
