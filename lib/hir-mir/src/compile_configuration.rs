use super::{
    concurrency_configuration::ConcurrencyConfiguration,
    error_type_configuration::ErrorTypeConfiguration,
    list_type_configuration::ListTypeConfiguration,
    string_type_configuration::StringTypeConfiguration,
};
#[cfg(test)]
use super::{
    concurrency_configuration::CONCURRENCY_CONFIGURATION,
    error_type_configuration::ERROR_TYPE_CONFIGURATION,
    list_type_configuration::LIST_TYPE_CONFIGURATION,
    string_type_configuration::STRING_TYPE_CONFIGURATION,
};
#[cfg(test)]
use once_cell::sync::Lazy;

#[derive(Clone, Debug)]
pub struct CompileConfiguration {
    pub list_type_configuration: ListTypeConfiguration,
    pub string_type_configuration: StringTypeConfiguration,
    pub error_type_configuration: ErrorTypeConfiguration,
    pub concurrency_configuration: ConcurrencyConfiguration,
}

#[cfg(test)]
pub static COMPILE_CONFIGURATION: Lazy<CompileConfiguration> = Lazy::new(|| CompileConfiguration {
    list_type_configuration: LIST_TYPE_CONFIGURATION.clone(),
    string_type_configuration: STRING_TYPE_CONFIGURATION.clone(),
    error_type_configuration: ERROR_TYPE_CONFIGURATION.clone(),
    concurrency_configuration: CONCURRENCY_CONFIGURATION.clone(),
});
