pub fn qualify(
    configuration: &hir_mir::CompileConfiguration,
    prelude_prefix: &str,
) -> hir_mir::CompileConfiguration {
    hir_mir::CompileConfiguration {
        error_type: qualify_error_type_configuration(&configuration.error_type, prelude_prefix),
        list_type: qualify_list_type_configuration(&configuration.list_type, prelude_prefix),
        string_type: qualify_string_type_configuration(&configuration.string_type, prelude_prefix),
        concurrency: configuration.concurrency.clone(),
    }
}

fn qualify_list_type_configuration(
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
        lazy_function_name: [prelude_prefix, &configuration.lazy_function_name].join(""),
        first_function_name: [prelude_prefix, &configuration.first_function_name].join(""),
        rest_function_name: [prelude_prefix, &configuration.rest_function_name].join(""),
        list_type_name: [prelude_prefix, &configuration.list_type_name].join(""),
        first_rest_type_name: [prelude_prefix, &configuration.first_rest_type_name].join(""),
    }
}

fn qualify_string_type_configuration(
    configuration: &hir_mir::StringTypeConfiguration,
    prelude_prefix: &str,
) -> hir_mir::StringTypeConfiguration {
    hir_mir::StringTypeConfiguration {
        equal_function_name: prelude_prefix.to_owned() + &configuration.equal_function_name,
    }
}

fn qualify_error_type_configuration(
    configuration: &hir_mir::ErrorTypeConfiguration,
    prelude_prefix: &str,
) -> hir_mir::ErrorTypeConfiguration {
    hir_mir::ErrorTypeConfiguration {
        error_type_name: prelude_prefix.to_owned() + &configuration.error_type_name,
        source_function_name: prelude_prefix.to_owned() + &configuration.source_function_name,
    }
}
