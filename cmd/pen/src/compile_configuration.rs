use once_cell::sync::Lazy;
use std::sync::Arc;

pub const CROSS_COMPILE_TARGETS: &[&str] = &[
    "i686-unknown-linux-musl",
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-musl",
    "wasm32-wasi",
];

pub static COMPILE_CONFIGURATION: Lazy<Arc<app::module_compiler::CompileConfiguration>> =
    Lazy::new(|| {
        app::module_compiler::CompileConfiguration {
            instruction: app::module_compiler::InstructionConfiguration {
                allocate_function_name: "_pen_malloc".into(),
                reallocate_function_name: "_pen_realloc".into(),
                free_function_name: "_pen_free".into(),
                unreachable_function_name: Some("_pen_unreachable".into()),
            },
            list_type: app::module_compiler::ListTypeConfiguration {
                empty_list_function_name: "_emptyList".into(),
                concatenate_function_name: "_concatenateLists".into(),
                equal_function_name: "_equalLists".into(),
                prepend_function_name: "_prependToList".into(),
                deconstruct_function_name: "_firstRest".into(),
                first_function_name: "_first".into(),
                rest_function_name: "_rest".into(),
                list_type_name: "_AnyList".into(),
                first_rest_type_name: "_FirstRest".into(),
            },
            string_type: app::module_compiler::StringTypeConfiguration {
                equal_function_name: "_equalStrings".into(),
            },
            error_type: app::module_compiler::ErrorTypeConfiguration {
                error_type_name: "error".into(),
            },
        }
        .into()
    });
