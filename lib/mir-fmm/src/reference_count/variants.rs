use super::{super::error::CompileError, expressions};
use crate::types;
use std::collections::BTreeMap;

pub fn compile_variant_clone_function(
    module_builder: &fmm::build::ModuleBuilder,
    type_: &mir::types::Type,
    types: &BTreeMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    module_builder.define_function(
        format!("variant_clone_{}", types::compile_type_id(type_)),
        vec![fmm::ir::Argument::new(
            "_payload",
            types::compile_variant_payload(),
        )],
        |builder| -> Result<_, CompileError> {
            let payload = fmm::build::variable("_payload", types::compile_variant_payload());

            expressions::clone_expression(
                &builder,
                &crate::variants::compile_unboxed_payload(&builder, &payload, type_, types)?,
                type_,
                types,
            )?;

            Ok(builder.return_(fmm::ir::VOID_VALUE.clone()))
        },
        fmm::types::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Weak,
    )
}

pub fn compile_variant_drop_function(
    module_builder: &fmm::build::ModuleBuilder,
    type_: &mir::types::Type,
    types: &BTreeMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    module_builder.define_function(
        format!("variant_drop_{}", types::compile_type_id(type_)),
        vec![fmm::ir::Argument::new(
            "_payload",
            types::compile_variant_payload(),
        )],
        |builder| -> Result<_, CompileError> {
            let payload = fmm::build::variable("_payload", types::compile_variant_payload());

            expressions::drop_expression(
                &builder,
                &crate::variants::compile_unboxed_payload(&builder, &payload, type_, types)?,
                type_,
                types,
            )?;

            Ok(builder.return_(fmm::ir::VOID_VALUE.clone()))
        },
        fmm::types::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
        fmm::ir::Linkage::Weak,
    )
}
