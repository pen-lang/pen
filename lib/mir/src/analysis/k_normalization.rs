mod conversion;

use crate::ir::*;
use conversion::convert;

pub fn normalize(module: &Module) -> Module {
    convert(module)
}
