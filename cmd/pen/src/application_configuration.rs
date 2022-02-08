use once_cell::sync::Lazy;
use std::sync::Arc;

pub static APPLICATION_CONFIGURATION: Lazy<Arc<app::ApplicationConfiguration>> = Lazy::new(|| {
    app::ApplicationConfiguration {
        application_filename: "app".into(),
        main_module_basename: "main".into(),
        main_function_module_basename: "MainFunction".into(),
        main_module: app::MainModuleConfiguration {
            source_main_function_name: "main".into(),
            object_main_function_name: "_pen_main".into(),
            main_function_type_name: "MainFunction".into(),
        },
    }
    .into()
});
