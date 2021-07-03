mod error;
pub mod type_canonicalizer;
pub mod type_comparability_checker;
pub mod type_equality_checker;
pub mod type_id_calculator;
pub mod type_resolver;
pub mod type_subsumption_checker;
pub mod union_difference_calculator;
pub mod union_type_creator;
pub mod union_type_member_calculator;

pub use error::TypeError;
