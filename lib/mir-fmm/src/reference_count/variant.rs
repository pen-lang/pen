use super::{super::error::CompileError, expression};
use crate::{context::Context, types};

const ARGUMENT_NAME: &str = "_payload";

pub fn compile_variant_clone_function(
    context: &Context,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    context.module_builder().define_function(
        format!("variant_clone_{}", types::compile_type_id(type_)),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            types::compile_variant_payload(),
        )],
        |builder| -> Result<_, CompileError> {
            Ok(builder.return_(crate::variant::compile_boxed_payload(
                &builder,
                &expression::clone_expression(
                    &builder,
                    &crate::variant::compile_unboxed_payload(
                        &builder,
                        &fmm::build::variable(ARGUMENT_NAME, types::compile_variant_payload()),
                        type_,
                        context.types(),
                    )?,
                    type_,
                    context.types(),
                )?,
            )?))
        },
        types::compile_variant_payload(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Weak,
    )
}

pub fn compile_variant_drop_function(
    context: &Context,
    type_: &mir::types::Type,
) -> Result<fmm::build::TypedExpression, CompileError> {
    context.module_builder().define_function(
        format!("variant_drop_{}", types::compile_type_id(type_)),
        vec![fmm::ir::Argument::new(
            ARGUMENT_NAME,
            types::compile_variant_payload(),
        )],
        |builder| -> Result<_, CompileError> {
            let payload = fmm::build::variable(ARGUMENT_NAME, types::compile_variant_payload());

            expression::drop_expression(
                &builder,
                &crate::variant::compile_unboxed_payload(
                    &builder,
                    &payload,
                    type_,
                    context.types(),
                )?,
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
