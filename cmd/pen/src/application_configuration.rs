use std::sync::Arc;
use std::sync::LazyLock;

pub static APPLICATION_CONFIGURATION: LazyLock<Arc<app::ApplicationConfiguration>> =
    LazyLock::new(|| {
        app::ApplicationConfiguration {
            application_filename: "app".into(),
            main_module_basename: "main".into(),
            context_module_basename: "Context".into(),
            main_module: app::MainModuleConfiguration {
                source_main_function_name: "main".into(),
                object_main_function_name: "_pen_main".into(),
                main_context_type_name: "context".into(),
                system_context_type_name: "Context".into(),
                new_system_context_function_name: "UnsafeNew".into(),
            },
        }
        .into()
    });
