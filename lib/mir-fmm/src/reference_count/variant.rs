use super::{super::error::CompileError, expression, pointer};
use crate::{context::Context, type_, variant};

const ARGUMENT_NAME: &str = "_payload";

pub fn compile_clone_function(
    context: &Context,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    context.module_builder().define_function(
        format!("variant_clone_{}", type_::compile_id(type_)),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            type_::compile_variant_payload(),
        )],
        |builder| -> Result<_, CompileError> {
            let payload = variant::bit_cast_from_opaque_payload(
                &builder,
                &fmm::build::variable(ARGUMENT_NAME, type_::compile_variant_payload()),
                type_,
                context.types(),
            )?;

            Ok(builder.return_(variant::bit_cast_to_opaque_payload(
                &builder,
                &if type_::variant::is_payload_boxed(type_, context.types())? {
                    pointer::clone(&builder, &payload)?
                } else {
                    expression::clone(&builder, &payload, type_, context.types())?
                },
            )?))
        },
        type_::compile_variant_payload(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Weak,
    )
}

pub fn compile_drop_function(
    context: &Context,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    context.module_builder().define_function(
        format!("variant_drop_{}", type_::compile_id(type_)),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            type_::compile_variant_payload(),
        )],
        |builder| -> Result<_, CompileError> {
            let payload = variant::bit_cast_from_opaque_payload(
                &builder,
                &fmm::build::variable(ARGUMENT_NAME, type_::compile_variant_payload()),
                type_,
                context.types(),
            )?;

            if type_::variant::is_payload_boxed(type_, context.types())? {
                pointer::drop(&builder, &payload, |builder| {
                    expression::drop(
                        builder,
                        &builder.load(payload.clone())?,
                        type_,
                        context.types(),
                    )
                })?
            } else {
                expression::drop(&builder, &payload, type_, context.types())?;
            }

            Ok(builder.return_(fmm::ir::VOID_VALUE.clone()))
        },
        fmm::types::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Weak,
    )
}

pub fn compile_synchronize_function(
    context: &Context,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    context.module_builder().define_function(
        format!("variant_synchronize_{}", type_::compile_id(type_)),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            type_::compile_variant_payload(),
        )],
        |builder| -> Result<_, CompileError> {
            let payload = variant::bit_cast_from_opaque_payload(
                &builder,
                &fmm::build::variable(ARGUMENT_NAME, type_::compile_variant_payload()),
                type_,
                context.types(),
            )?;

            if type_::variant::is_payload_boxed(type_, context.types())? {
                pointer::synchronize(&builder, &payload, |builder| {
                    expression::synchronize(
                        builder,
                        &builder.load(payload.clone())?,
                        type_,
                        context.types(),
                    )
                })?;
            } else {
                expression::synchronize(&builder, &payload, type_, context.types())?;
            }

            Ok(builder.return_(fmm::ir::VOID_VALUE.clone()))
        },
        fmm::types::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Weak,
    )
}
