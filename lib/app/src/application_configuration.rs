pub struct ApplicationConfiguration {
    pub application_filename: String,
    pub main_module_basename: String,
    pub main_function_module_basename: String,
    pub main_module: MainModuleConfiguration,
}

pub type MainModuleConfiguration = hir_mir::MainModuleConfiguration;
