pub mod block;
mod count;
mod expression;
mod function;
pub mod heap;
pub mod pointer;
pub mod record;
pub mod variant;

pub use expression::*;
use once_cell::sync::Lazy;

pub(super) static REFERENCE_COUNT_FUNCTION_DEFINITION_OPTIONS: Lazy<
    fmm::ir::FunctionDefinitionOptions,
> = Lazy::new(|| {
    fmm::ir::FunctionDefinitionOptions::new()
        .set_address_named(false)
        .set_calling_convention(fmm::types::CallingConvention::Target)
        .set_linkage(fmm::ir::Linkage::Weak)
});
