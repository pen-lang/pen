mod context;
pub mod duplicate_function_name_validator;
pub mod duplicate_type_name_validator;
mod error;
pub mod expression_visitor;
pub mod function_definition_qualifier;
pub mod module_environment_creator;
pub mod record_field_resolver;
pub mod record_field_validator;
pub mod recursive_type_alias_validator;
pub mod try_operation_validator;
pub mod type_canonicalizer;
pub mod type_checker;
pub mod type_coercer;
pub mod type_collector;
pub mod type_comparability_checker;
pub mod type_difference_calculator;
pub mod type_equality_checker;
pub mod type_existence_validator;
pub mod type_extractor;
pub mod type_formatter;
pub mod type_id_calculator;
pub mod type_inferrer;
pub mod type_qualifier;
pub mod type_resolver;
pub mod type_subsumption_checker;
pub mod type_transformer;
pub mod union_type_creator;
pub mod union_type_member_calculator;
pub mod unused_error_validator;
pub mod variable_renamer;
pub mod variable_transformer;

use crate::ir::Module;
pub use context::AnalysisContext;
pub use error::AnalysisError;

// Validate a module and replace subtyping with type coercion there.
pub fn analyze(context: &AnalysisContext, module: &Module) -> Result<Module, AnalysisError> {
    duplicate_function_name_validator::validate(module)?;
    duplicate_type_name_validator::validate(module)?;
    type_existence_validator::validate(
        module,
        &context.types().keys().cloned().collect(),
        &context.records().keys().cloned().collect(),
    )?;
    recursive_type_alias_validator::validate(module)?;

    let module = type_inferrer::infer(context, module)?;
    type_checker::check_types(context, &module)?;

    try_operation_validator::validate(context, &module)?;
    record_field_validator::validate(context, &module)?;
    unused_error_validator::validate(context, &module)?;

    let module = type_coercer::coerce_types(context, &module)?;
    type_checker::check_types(context, &module)?;

    Ok(module)
}
