#[cfg(test)]
use std::sync::LazyLock;

#[cfg(test)]
pub static STRING_TYPE_CONFIGURATION: LazyLock<StringTypeConfiguration> =
    LazyLock::new(|| StringTypeConfiguration {
        equal_function_name: "_equalStrings".into(),
    });

#[derive(Clone, Debug)]
pub struct StringTypeConfiguration {
    pub equal_function_name: String,
}
