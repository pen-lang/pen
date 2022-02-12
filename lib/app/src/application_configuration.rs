pub struct ApplicationConfiguration {
    pub application_filename: String,
    pub main_module_basename: String,
    pub context_module_basename: String,
    pub main_module: MainModuleConfiguration,
}

pub struct MainModuleConfiguration {
    pub source_main_function_name: String,
    pub object_main_function_name: String,
    pub context_type_name: String,
    pub new_context_function_name: String,
}
