mod environment_inference;
mod free_variable;
pub mod lambda_lifting;
mod reference_count;
mod type_check;
mod variant_type_collection;

pub use environment_inference::*;
pub use free_variable::*;
pub use reference_count::*;
pub use type_check::*;
pub use variant_type_collection::*;
