mod conversion;
mod error;
mod heap_block_set;

use self::{conversion::convert, error::ReuseError};
use crate::ir::*;

pub fn reuse_heap(module: &Module) -> Result<Module, ReuseError> {
    convert(module)
}
