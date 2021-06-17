mod error;
mod type_canonicalizer;
mod type_equality_checker;
mod type_resolver;
mod type_subsumption_checker;

pub use error::TypeAnalysisError;
pub use type_canonicalizer::canonicalize;
pub use type_equality_checker::check_equality;
pub use type_resolver::{
    resolve_record_elements, resolve_to_function, resolve_to_record, resolve_type,
};
pub use type_subsumption_checker::check_subsumption;
