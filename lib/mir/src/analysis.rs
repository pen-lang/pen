mod environment_inference;
mod free_variables;
mod heap_reuse;
mod k_normalization;
mod reference_count;
mod type_check;
mod variant_type_collection;

pub use environment_inference::*;
pub use free_variables::*;
pub use heap_reuse::*;
pub use k_normalization::*;
pub use reference_count::*;
pub use type_check::*;
pub use variant_type_collection::*;
