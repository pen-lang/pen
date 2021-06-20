mod error;
pub mod type_canonicalizer;
pub mod type_equality_checker;
pub mod type_id_calculator;
pub mod type_resolver;
pub mod type_subsumption_checker;

// TODO Remove those re-exports.
pub use error::TypeAnalysisError;
pub use type_canonicalizer::canonicalize;
pub use type_equality_checker::check_equality;
pub use type_resolver::{
    resolve_record_elements, resolve_to_function, resolve_to_record, resolve_type,
};
pub use type_subsumption_checker::check_subsumption;
