mod error;
mod transformation;
mod validation;

use crate::ir::Module;
pub use error::ReferenceCountError;
use validation::validate;

pub fn transform(module: &Module) -> Result<Module, ReferenceCountError> {
    let module = transformation::transform(module)?;

    validate(&module)?;

    Ok(module)
}
