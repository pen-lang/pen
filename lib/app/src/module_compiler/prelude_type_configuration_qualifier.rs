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
        empty_list_function_name: prelude_prefix.to_owned()
            + &configuration.empty_list_function_name,
        concatenate_function_name: prelude_prefix.to_owned()
            + &configuration.concatenate_function_name,
        equal_function_name: prelude_prefix.to_owned() + &configuration.equal_function_name,
        prepend_function_name: prelude_prefix.to_owned() + &configuration.prepend_function_name,
        deconstruct_function_name: prelude_prefix.to_owned()
            + &configuration.deconstruct_function_name,
        lazy_function_name: prelude_prefix.to_owned() + &configuration.lazy_function_name,
        first_function_name: prelude_prefix.to_owned() + &configuration.first_function_name,
        rest_function_name: prelude_prefix.to_owned() + &configuration.rest_function_name,
        list_type_name: prelude_prefix.to_owned() + &configuration.list_type_name,
        first_rest_type_name: prelude_prefix.to_owned() + &configuration.first_rest_type_name,
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
