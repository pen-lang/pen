mod conversion;
mod error;
mod validation;

use crate::ir::Module;
use conversion::convert_module;
pub use error::ReferenceCountError;
use validation::validate;

pub fn count_references(module: &Module) -> Result<Module, ReferenceCountError> {
    let module = convert_module(module)?;

    validate(&module)?;

    Ok(module)
}
