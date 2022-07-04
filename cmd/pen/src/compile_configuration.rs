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
            fmm: app::module_compiler::FmmConfiguration {
                allocate_function_name: "_pen_malloc".into(),
                reallocate_function_name: "_pen_realloc".into(),
                free_function_name: "_pen_free".into(),
                unreachable_function_name: Some("_pen_unreachable".into()),
            },
            mir: app::module_compiler::MirConfiguration {
                yield_function_name: "_pen_yield".into(),
            },
            hir: app::module_compiler::HirConfiguration {
                list_type: app::module_compiler::ListTypeConfiguration {
                    empty_list_function_name: "_emptyList".into(),
                    concatenate_function_name: "_concatenateLists".into(),
                    equal_function_name: "_equalLists".into(),
                    prepend_function_name: "_prependToList".into(),
                    deconstruct_function_name: "_firstRest".into(),
                    lazy_function_name: "_lazy".into(),
                    first_function_name: "_first".into(),
                    rest_function_name: "_rest".into(),
                    list_type_name: "_anyList".into(),
                    first_rest_type_name: "_firstRest".into(),
                    size_function_name: "_listSize".into(),
                },
                map_type: app::module_compiler::MapTypeConfiguration {
                    context_function_name: "_newMapContext".into(),
                    context_type_name: "_mapContext".into(),
                    empty_function_name: "_newMap".into(),
                    empty_type_name: "_empty".into(),
                    equal_function_name: "_equalMaps".into(),
                    get_function_name: "_getMap".into(),
                    map_type_name: "_map".into(),
                    merge_function_name: "_mergeMaps".into(),
                    delete_function_name: "_deleteMap".into(),
                    set_function_name: "_setMap".into(),
                    size_function_name: "_mapSize".into(),
                    hash: app::module_compiler::HashConfiguration {
                        combine_function_name: "_combineHashes".into(),
                        number_hash_function_name: "_hashNumber".into(),
                        string_hash_function_name: "_hashString".into(),
                        list_hash_function_name: "_hashList".into(),
                        map_hash_function_name: "_hashMap".into(),
                    },
                    iteration: app::module_compiler::MapTypeIterationConfiguration {
                        iterator_type_name: "_mapIterator".into(),
                        iterate_function_name: "_iterateMap".into(),
                        key_function_name: "_mapIteratorKey".into(),
                        value_function_name: "_mapIteratorValue".into(),
                        rest_function_name: "_mapIteratorRest".into(),
                    },
                },
                string_type: app::module_compiler::StringTypeConfiguration {
                    equal_function_name: "_equalStrings".into(),
                },
                error_type: app::module_compiler::ErrorTypeConfiguration {
                    error_type_name: "error".into(),
                    error_function_name: "error".into(),
                    source_function_name: "source".into(),
                },
                spawn_function_name: "_pen_spawn".into(),
                debug_function_name: "_pen_debug".into(),
            },
        }
        .into()
    });
