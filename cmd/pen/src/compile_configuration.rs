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
                    empty_list_function_name: "EmptyList".into(),
                    concatenate_function_name: "ConcatenateLists".into(),
                    equal_function_name: "EqualLists".into(),
                    prepend_function_name: "PrependToList".into(),
                    deconstruct_function_name: "FirstRest".into(),
                    lazy_function_name: "LazyList".into(),
                    first_function_name: "First".into(),
                    rest_function_name: "Rest".into(),
                    list_type_name: "List".into(),
                    first_rest_type_name: "FirstRest".into(),
                    size_function_name: "ListSize".into(),
                },
                map_type: app::module_compiler::MapTypeConfiguration {
                    context_function_name: "NewMapContext".into(),
                    context_type_name: "MapContext".into(),
                    empty_function_name: "NewMap".into(),
                    empty_type_name: "Empty".into(),
                    equal_function_name: "EqualMaps".into(),
                    get_function_name: "GetMap".into(),
                    map_type_name: "Map".into(),
                    merge_function_name: "MergeMaps".into(),
                    delete_function_name: "DeleteMap".into(),
                    set_function_name: "SetMap".into(),
                    size_function_name: "MapSize".into(),
                    hash: app::module_compiler::HashConfiguration {
                        combine_function_name: "CombineHashes".into(),
                        number_hash_function_name: "HashNumber".into(),
                        string_hash_function_name: "HashString".into(),
                        list_hash_function_name: "HashList".into(),
                        map_hash_function_name: "HashMap".into(),
                    },
                    iteration: app::module_compiler::MapTypeIterationConfiguration {
                        iterator_type_name: "MapIterator".into(),
                        iterate_function_name: "IterateMap".into(),
                        key_function_name: "MapIteratorKey".into(),
                        value_function_name: "MapIteratorValue".into(),
                        rest_function_name: "MapIteratorRest".into(),
                    },
                },
                string_type: app::module_compiler::StringTypeConfiguration {
                    equal_function_name: "EqualStrings".into(),
                },
                error_type: app::module_compiler::ErrorTypeConfiguration {
                    error_type_name: "Error".into(),
                    error_function_name: "Error".into(),
                    source_function_name: "Source".into(),
                },
                spawn_function_name: "_pen_spawn".into(),
                debug_function_name: "_pen_debug".into(),
            },
        }
        .into()
    });
