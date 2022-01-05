use once_cell::sync::Lazy;

#[cfg(test)]
pub static STRING_TYPE_CONFIGURATION: Lazy<StringTypeConfiguration> =
    Lazy::new(|| StringTypeConfiguration {
        equal_function_name: "_equalStrings".into(),
    });

pub static DUMMY_STRING_TYPE_CONFIGURATION: Lazy<StringTypeConfiguration> =
    Lazy::new(|| StringTypeConfiguration {
        equal_function_name: "<dummy>".into(),
    });

#[derive(Clone, Debug)]
pub struct StringTypeConfiguration {
    pub equal_function_name: String,
}
