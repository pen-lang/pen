pub mod expression;
pub mod module;

use crate::CompileError;
use fnv::FnvHashMap;
use hir::{
    analysis::type_id_calculator,
    types::{self, Type},
};

fn context_function_name(
    type_: &types::Map,
    types: &FnvHashMap<String, Type>,
) -> Result<String, CompileError> {
    Ok(format!(
        "hir:map:context:{}",
        type_id_calculator::calculate(&type_.clone().into(), types)?
    ))
}
