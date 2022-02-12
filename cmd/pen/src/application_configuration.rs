use once_cell::sync::Lazy;
use std::sync::Arc;

pub static APPLICATION_CONFIGURATION: Lazy<Arc<app::ApplicationConfiguration>> = Lazy::new(|| {
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
