pub fn qualify_list_type_configuration(
    configuration: &hir_mir::ListTypeConfiguration,
    prelude_prefix: &str,
) -> hir_mir::ListTypeConfiguration {
    hir_mir::ListTypeConfiguration {
        empty_list_function_name: [prelude_prefix, &configuration.empty_list_function_name]
            .join(""),
        concatenate_function_name: [prelude_prefix, &configuration.concatenate_function_name]
            .join(""),
        equal_function_name: [prelude_prefix, &configuration.equal_function_name].join(""),
        prepend_function_name: [prelude_prefix, &configuration.prepend_function_name].join(""),
        deconstruct_function_name: [prelude_prefix, &configuration.deconstruct_function_name]
            .join(""),
        first_function_name: [prelude_prefix, &configuration.first_function_name].join(""),
        rest_function_name: [prelude_prefix, &configuration.rest_function_name].join(""),
        list_type_name: [prelude_prefix, &configuration.list_type_name].join(""),
        first_rest_type_name: [prelude_prefix, &configuration.first_rest_type_name].join(""),
    }
}

pub fn qualify_string_type_configuration(
    configuration: &hir_mir::StringTypeConfiguration,
    prelude_prefix: &str,
) -> hir_mir::StringTypeConfiguration {
    hir_mir::StringTypeConfiguration {
        equal_function_name: prelude_prefix.to_owned() + &configuration.equal_function_name,
    }
}

pub fn qualify_error_type_configuration(
    configuration: &hir_mir::ErrorTypeConfiguration,
    prelude_prefix: &str,
) -> hir_mir::ErrorTypeConfiguration {
    hir_mir::ErrorTypeConfiguration {
        error_type_name: prelude_prefix.to_owned() + &configuration.error_type_name,
    }
}
