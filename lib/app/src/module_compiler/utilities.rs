use once_cell::sync::Lazy;

pub static DUMMY_LIST_TYPE_CONFIGURATION: Lazy<lang::hir_mir::ListTypeConfiguration> =
    Lazy::new(|| lang::hir_mir::ListTypeConfiguration {
        empty_list_function_name: "<dummy>".into(),
        concatenate_function_name: "<dummy>".into(),
        equal_function_name: "<dummy>".into(),
        prepend_function_name: "<dummy>".into(),
        deconstruct_function_name: "<dummy>".into(),
        first_function_name: "<dummy>".into(),
        rest_function_name: "<dummy>".into(),
        list_type_name: "<dummy>".into(),
        first_rest_type_name: "<dummy>".into(),
        map_function_name: "<dummy>".into(),
    });

pub static DUMMY_STRING_TYPE_CONFIGURATION: Lazy<lang::hir_mir::StringTypeConfiguration> =
    Lazy::new(|| lang::hir_mir::StringTypeConfiguration {
        equal_function_name: "<dummy>".into(),
    });

pub static DUMMY_ERROR_TYPE_CONFIGURATION: Lazy<lang::hir_mir::ErrorTypeConfiguration> =
    Lazy::new(|| lang::hir_mir::ErrorTypeConfiguration {
        error_type_name: "<dummy>".into(),
    });
