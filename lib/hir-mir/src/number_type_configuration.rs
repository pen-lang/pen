#[cfg(test)]
use once_cell::sync::Lazy;

#[cfg(test)]
pub static NUMBER_TYPE_CONFIGURATION: Lazy<NumberTypeConfiguration> =
    Lazy::new(|| NumberTypeConfiguration {
        debug_function_name: "debugNumber".into(),
    });

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumberTypeConfiguration {
    pub debug_function_name: String,
}
