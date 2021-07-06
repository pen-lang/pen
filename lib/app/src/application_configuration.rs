pub struct ApplicationConfiguration {
    pub main_module_basename: String,
    pub main_module: MainModuleConfiguration,
}

pub type MainModuleConfiguration = lang::hir_mir::MainModuleConfiguration;
