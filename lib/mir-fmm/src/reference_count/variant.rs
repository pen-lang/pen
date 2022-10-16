use super::{
    super::error::CompileError, expression, pointer, REFERENCE_COUNT_FUNCTION_DEFINITION_OPTIONS,
};
use crate::{context::Context, type_, variant};

const FUNCTION_PREFIX: &str = "mir:variant:";
const ARGUMENT_NAME: &str = "_payload";

pub fn compile_clone_function(
    context: &Context,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    context.module_builder().define_function(
        compile_function_name(type_, "clone"),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            type_::compile_variant_payload(),
        )],
        type_::compile_variant_payload(),
        |builder| -> Result<_, CompileError> {
            let payload = variant::bit_cast_from_opaque_payload(
                context,
                &builder,
                &fmm::build::variable(ARGUMENT_NAME, type_::compile_variant_payload()),
                type_,
            )?;

            Ok(builder.return_(variant::bit_cast_to_opaque_payload(
                &builder,
                &if type_::variant::is_payload_boxed(context, type_)? {
                    pointer::clone(&builder, &payload)?
                } else {
                    expression::clone(context, &builder, &payload, type_)?
                },
            )?))
        },
        REFERENCE_COUNT_FUNCTION_DEFINITION_OPTIONS.clone(),
    )
}

pub fn compile_drop_function(
    context: &Context,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    context.module_builder().define_function(
        compile_function_name(type_, "drop"),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            type_::compile_variant_payload(),
        )],
        fmm::types::void_type(),
        |builder| -> Result<_, CompileError> {
            let payload = variant::bit_cast_from_opaque_payload(
                context,
                &builder,
                &fmm::build::variable(ARGUMENT_NAME, type_::compile_variant_payload()),
                type_,
            )?;

            if type_::variant::is_payload_boxed(context, type_)? {
                pointer::drop(&builder, &payload, |builder| {
                    expression::drop(context, builder, &builder.load(payload.clone())?, type_)
                })?
            } else {
                expression::drop(context, &builder, &payload, type_)?;
            }

            Ok(builder.return_(fmm::ir::void_value()))
        },
        REFERENCE_COUNT_FUNCTION_DEFINITION_OPTIONS.clone(),
    )
}

pub fn compile_synchronize_function(
    context: &Context,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    context.module_builder().define_function(
        compile_function_name(type_, "synchronize"),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            type_::compile_variant_payload(),
        )],
        fmm::types::void_type(),
        |builder| -> Result<_, CompileError> {
            let payload = variant::bit_cast_from_opaque_payload(
                context,
                &builder,
                &fmm::build::variable(ARGUMENT_NAME, type_::compile_variant_payload()),
                type_,
            )?;

            if type_::variant::is_payload_boxed(context, type_)? {
                pointer::synchronize(&builder, &payload, |builder| {
                    expression::synchronize(
                        context,
                        builder,
                        &builder.load(payload.clone())?,
                        type_,
                    )
                })?;
            } else {
                expression::synchronize(context, &builder, &payload, type_)?;
            }

            Ok(builder.return_(fmm::ir::void_value()))
        },
        REFERENCE_COUNT_FUNCTION_DEFINITION_OPTIONS.clone(),
    )
}

fn compile_function_name(type_: &mir::types::Type, operation: &str) -> String {
    format!(
        "{}:{}:{}",
        FUNCTION_PREFIX,
        operation,
        mir::analysis::type_id::calculate(type_)
    )
}
