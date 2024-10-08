#[cfg(test)]
use std::sync::LazyLock;

#[cfg(test)]
pub static NUMBER_TYPE_CONFIGURATION: LazyLock<NumberTypeConfiguration> =
    LazyLock::new(|| NumberTypeConfiguration {
        debug_function_name: "debugNumber".into(),
    });

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumberTypeConfiguration {
    pub debug_function_name: String,
}
