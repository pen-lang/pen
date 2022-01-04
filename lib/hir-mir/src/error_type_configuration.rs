use once_cell::sync::Lazy;

#[cfg(test)]
pub static ERROR_TYPE_CONFIGURATION: Lazy<ErrorTypeConfiguration> =
    Lazy::new(|| ErrorTypeConfiguration {
        error_type_name: "error".into(),
    });

pub static DUMMY_ERROR_TYPE_CONFIGURATION: Lazy<ErrorTypeConfiguration> =
    Lazy::new(|| ErrorTypeConfiguration {
        error_type_name: "<dummy>".into(),
    });

#[derive(Clone, Debug, PartialEq)]
pub struct ErrorTypeConfiguration {
    pub error_type_name: String,
}
