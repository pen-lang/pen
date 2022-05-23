use crate::{box_, context::Context, type_, CompileError};
use fnv::FnvHashMap;

const VARIANT_TAG_FIELD_INDEX: usize = 0;
const VARIANT_PAYLOAD_FIELD_INDEX: usize = 1;

pub fn compile_tag(type_: &mir::types::Type) -> fmm::build::TypedExpression {
    fmm::build::variable(type_::compile_id(type_), type_::compile_variant_tag())
}

pub fn extract_tag(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.deconstruct_record(expression.clone(), VARIANT_TAG_FIELD_INDEX)?)
}

pub fn extract_payload(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.deconstruct_record(expression.clone(), VARIANT_PAYLOAD_FIELD_INDEX)?)
}

pub fn bit_cast_to_opaque_payload(
    builder: &fmm::build::InstructionBuilder,
    payload: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(compile_payload_bit_cast(
        builder,
        type_::compile_variant_payload(),
        payload.clone(),
    )?)
}

pub fn bit_cast_from_opaque_payload(
    builder: &fmm::build::InstructionBuilder,
    payload: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(compile_payload_bit_cast(
        builder,
        type_::variant::compile_payload(type_, types)?,
        payload.clone(),
    )?)
}

pub fn downcast(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    variant: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    unbox_payload(
        context,
        builder,
        &bit_cast_from_opaque_payload(
            &builder,
            &extract_payload(&builder, &variant)?,
            type_,
            context.types(),
        )?,
        type_,
    )
}

pub fn box_payload(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    value: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(
        if type_::variant::should_box_payload(type_, context.types())? {
            box_::box_(builder, value.clone())?
        } else {
            value.clone()
        },
    )
}

fn unbox_payload(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    value: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(
        if type_::variant::should_box_payload(type_, context.types())? {
            box_::unbox(builder, value.clone(), type_, context.types())?
        } else {
            value.clone()
        },
    )
}

fn compile_payload_bit_cast(
    builder: &fmm::build::InstructionBuilder,
    to_type: impl Into<fmm::types::Type>,
    argument: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    let argument = argument.into();
    let to_type = to_type.into();

    Ok(if argument.type_() == &to_type {
        argument
    } else {
        builder.deconstruct_union(
            fmm::ir::Union::new(
                fmm::types::Union::new(vec![argument.type_().clone(), to_type]),
                0,
                argument.expression().clone(),
            ),
            1,
        )?
    })
}
