pub fn qualify(
    configuration: &hir_mir::CompileConfiguration,
    prelude_prefix: &str,
) -> hir_mir::CompileConfiguration {
    hir_mir::CompileConfiguration {
        error_type: qualify_error_type_configuration(&configuration.error_type, prelude_prefix),
        list_type: qualify_list_type_configuration(&configuration.list_type, prelude_prefix),
        map_type: qualify_map_type_configuration(&configuration.map_type, prelude_prefix),
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
        size_function_name: prelude_prefix.to_owned() + &configuration.size_function_name,
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

fn qualify_map_type_configuration(
    configuration: &hir_mir::MapTypeConfiguration,
    prelude_prefix: &str,
) -> hir_mir::MapTypeConfiguration {
    hir_mir::MapTypeConfiguration {
        context_function_name: prelude_prefix.to_owned() + &configuration.context_function_name,
        context_type_name: prelude_prefix.to_owned() + &configuration.context_type_name,
        empty_function_name: prelude_prefix.to_owned() + &configuration.empty_function_name,
        equal_function_name: prelude_prefix.to_owned() + &configuration.equal_function_name,
        get_function_name: prelude_prefix.to_owned() + &configuration.get_function_name,
        map_type_name: prelude_prefix.to_owned() + &configuration.map_type_name,
        merge_function_name: prelude_prefix.to_owned() + &configuration.merge_function_name,
        empty_type_name: prelude_prefix.to_owned() + &configuration.empty_type_name,
        delete_function_name: prelude_prefix.to_owned() + &configuration.delete_function_name,
        set_function_name: prelude_prefix.to_owned() + &configuration.set_function_name,
        size_function_name: prelude_prefix.to_owned() + &configuration.size_function_name,
        hash: qualify_hash_configuration(&configuration.hash, prelude_prefix),
        iteration: qualify_map_type_iteration_configuration(
            &configuration.iteration,
            prelude_prefix,
        ),
    }
}

fn qualify_hash_configuration(
    configuration: &hir_mir::HashConfiguration,
    prelude_prefix: &str,
) -> hir_mir::HashConfiguration {
    hir_mir::HashConfiguration {
        combine_function_name: prelude_prefix.to_owned() + &configuration.combine_function_name,
        number_hash_function_name: prelude_prefix.to_owned()
            + &configuration.number_hash_function_name,
        string_hash_function_name: prelude_prefix.to_owned()
            + &configuration.string_hash_function_name,
        list_hash_function_name: prelude_prefix.to_owned() + &configuration.list_hash_function_name,
        map_hash_function_name: prelude_prefix.to_owned() + &configuration.map_hash_function_name,
    }
}

fn qualify_map_type_iteration_configuration(
    configuration: &hir_mir::MapTypeIterationConfiguration,
    prelude_prefix: &str,
) -> hir_mir::MapTypeIterationConfiguration {
    hir_mir::MapTypeIterationConfiguration {
        iterator_type_name: prelude_prefix.to_owned() + &configuration.iterator_type_name,
        iterate_function_name: prelude_prefix.to_owned() + &configuration.iterate_function_name,
        key_function_name: prelude_prefix.to_owned() + &configuration.key_function_name,
        value_function_name: prelude_prefix.to_owned() + &configuration.value_function_name,
        rest_function_name: prelude_prefix.to_owned() + &configuration.rest_function_name,
    }
}
