#[cfg(test)]
use once_cell::sync::Lazy;

#[cfg(test)]
pub static ERROR_TYPE_CONFIGURATION: Lazy<ErrorTypeConfiguration> =
    Lazy::new(|| ErrorTypeConfiguration {
        error_type_name: "error".into(),
        error_function_name: "error".into(),
        source_function_name: "source".into(),
    });

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErrorTypeConfiguration {
    pub error_type_name: String,
    pub error_function_name: String,
    pub source_function_name: String,
}
