use crate::{CompileError, box_, context::Context, pointer, type_, type_information};

const VARIANT_TAG_FIELD_INDEX: usize = 0;
const VARIANT_PAYLOAD_FIELD_INDEX: usize = 1;

fn compile_tag(type_: &mir::types::Type) -> fmm::build::TypedExpression {
    fmm::build::variable(
        type_information::get_global_variable_name(type_),
        type_::compile_variant_tag(),
    )
}

pub fn compile_tag_comparison(
    builder: &fmm::build::InstructionBuilder,
    argument: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(pointer::equal(
        get_tag(builder, argument)?,
        compile_tag(type_),
    )?)
}

pub fn get_tag(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.deconstruct_record(expression.clone(), VARIANT_TAG_FIELD_INDEX)?)
}

pub fn get_payload(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.deconstruct_record(expression.clone(), VARIANT_PAYLOAD_FIELD_INDEX)?)
}

pub fn bit_cast_to_opaque_payload(
    builder: &fmm::build::InstructionBuilder,
    payload: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(bit_cast_payload(
        builder,
        type_::compile_variant_payload(),
        payload.clone(),
    )?)
}

pub fn bit_cast_from_opaque_payload(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    payload: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(bit_cast_payload(
        builder,
        type_::variant::compile_payload(context, type_)?,
        payload.clone(),
    )?)
}

pub fn upcast(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    value: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::record(vec![
        compile_tag(type_),
        bit_cast_to_opaque_payload(builder, &box_payload(context, builder, value, type_)?)?,
    ])
    .into())
}

pub fn downcast(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    variant: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(if matches!(type_, mir::types::Type::Variant) {
        variant.clone()
    } else {
        unbox_payload(
            context,
            builder,
            &bit_cast_from_opaque_payload(
                context,
                builder,
                &get_payload(builder, variant)?,
                type_,
            )?,
            type_,
        )?
    })
}

fn box_payload(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    value: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(if type_::variant::is_payload_boxed(context, type_)? {
        box_::box_(builder, value.clone())?
    } else {
        value.clone()
    })
}

fn unbox_payload(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    value: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(if type_::variant::is_payload_boxed(context, type_)? {
        box_::unbox(context, builder, value.clone(), type_)?
    } else {
        value.clone()
    })
}

fn bit_cast_payload(
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
