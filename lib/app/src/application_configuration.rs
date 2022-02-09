pub struct ApplicationConfiguration {
    pub application_filename: String,
    pub main_module_basename: String,
    pub context_module_basename: String,
    pub main_module: MainModuleConfiguration,
}

pub type MainModuleConfiguration = hir_mir::MainModuleConfiguration;
