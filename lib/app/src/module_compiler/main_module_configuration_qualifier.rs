use crate::application_configuration::MainModuleConfiguration;

pub fn qualify(
    configuration: &MainModuleConfiguration,
    main_function_module_prefix: &str,
) -> MainModuleConfiguration {
    MainModuleConfiguration {
        source_main_function_name: configuration.source_main_function_name.clone(),
        object_main_function_name: configuration.object_main_function_name.clone(),
        main_function_type_name: main_function_module_prefix.to_owned()
            + &configuration.main_function_type_name,
    }
}
