use crate::context::CompileContext;
use crate::error::CompileError;
use fnv::FnvHashMap;
use hir::ir::*;
use hir::{analysis::type_id_calculator, types::Type};

pub fn transform(context: &CompileContext, type_: &Type) -> Result<Expression, CompileError> {
    Ok(Variable::new(
        transform_name(type_, context.types())?,
        type_.position().clone(),
    )
    .into())
}

pub fn transform_name(
    type_: &Type,
    types: &FnvHashMap<String, Type>,
) -> Result<String, CompileError> {
    Ok(format!(
        "hir:equal:{}",
        type_id_calculator::calculate(&type_.clone(), types)?
    ))
}
