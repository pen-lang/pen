use super::{super::error::CompileError, expression};
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
            Ok(builder.return_(variant::compile_boxed_payload(
                &builder,
                &expression::clone(
                    &builder,
                    &variant::compile_unboxed_payload(
                        &builder,
                        &fmm::build::variable(ARGUMENT_NAME, type_::compile_variant_payload()),
                        type_,
                        context.types(),
                    )?,
                    type_,
                    context.types(),
                )?,
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
            let payload = fmm::build::variable(ARGUMENT_NAME, type_::compile_variant_payload());

            expression::drop(
                &builder,
                &variant::compile_unboxed_payload(&builder, &payload, type_, context.types())?,
                type_,
                context.types(),
            )?;

            Ok(builder.return_(fmm::ir::VOID_VALUE.clone()))
        },
        fmm::types::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Weak,
    )
}
