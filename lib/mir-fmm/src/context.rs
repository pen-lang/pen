use crate::configuration::Configuration;
use fnv::FnvHashMap;

pub struct Context {
    module_builder: fmm::build::ModuleBuilder,
    types: FnvHashMap<String, mir::types::RecordBody>,
    configuration: Configuration,
}

impl Context {
    pub fn new(module: &mir::ir::Module, configuration: Configuration) -> Self {
        Self {
            module_builder: fmm::build::ModuleBuilder::new(),
            types: module
                .type_definitions()
                .iter()
                .map(|definition| (definition.name().into(), definition.type_().clone()))
                .collect(),
            configuration,
        }
    }

    pub fn module_builder(&self) -> &fmm::build::ModuleBuilder {
        &self.module_builder
    }

    pub fn types(&self) -> &FnvHashMap<String, mir::types::RecordBody> {
        &self.types
    }

    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }
}
