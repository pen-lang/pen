use super::{ErrorTypeConfiguration, ListTypeConfiguration, StringTypeConfiguration};
use once_cell::sync::Lazy;

pub static DUMMY_LIST_TYPE_CONFIGURATION: Lazy<ListTypeConfiguration> =
    Lazy::new(|| ListTypeConfiguration {
        empty_list_function_name: "<dummy>".into(),
        concatenate_function_name: "<dummy>".into(),
        equal_function_name: "<dummy>".into(),
        prepend_function_name: "<dummy>".into(),
        deconstruct_function_name: "<dummy>".into(),
        first_function_name: "<dummy>".into(),
        rest_function_name: "<dummy>".into(),
        list_type_name: "<dummy>".into(),
        first_rest_type_name: "<dummy>".into(),
    });

pub static DUMMY_STRING_TYPE_CONFIGURATION: Lazy<StringTypeConfiguration> =
    Lazy::new(|| StringTypeConfiguration {
        equal_function_name: "<dummy>".into(),
    });

pub static DUMMY_ERROR_TYPE_CONFIGURATION: Lazy<ErrorTypeConfiguration> =
    Lazy::new(|| ErrorTypeConfiguration {
        error_type_name: "<dummy>".into(),
    });
