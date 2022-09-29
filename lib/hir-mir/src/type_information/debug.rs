use crate::{context::CompileContext, CompileError};
use hir::{analysis::type_id_calculator, types::Type};

pub fn compile_function_name(
    context: &CompileContext,
    type_: &Type,
) -> Result<String, CompileError> {
    Ok(format!(
        "hir:debug:{}",
        type_id_calculator::calculate(type_, context.types())?
    ))
}
