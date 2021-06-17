use once_cell::sync::Lazy;
use std::sync::Arc;

pub static COMPILE_CONFIGURATION: Lazy<Arc<app::compile::CompileConfiguration>> = Lazy::new(|| {
    app::compile::CompileConfiguration {
        heap: app::compile::HeapConfiguration {
            allocate_function_name: "malloc".into(),
            reallocate_function_name: "realloc".into(),
            free_function_name: "free".into(),
        },
        list_type: app::compile::ListTypeConfiguration {
            empty_list_variable_name: "_emptyList".into(),
            concatenate_function_name: "_concatenateLists".into(),
            equal_function_name: "_equalLists".into(),
            prepend_function_name: "_prependToList".into(),
            deconstruct_function_name: "_firstRest".into(),
            first_function_name: "_first".into(),
            rest_function_name: "_rest".into(),
            list_type_name: "_AnyList".into(),
            first_rest_type_name: "_FirstRest".into(),
            map_function_name: "_mapList".into(),
        },
    }
    .into()
});
