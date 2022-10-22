use crate::configuration::Configuration;
use fnv::FnvHashMap;
use std::cell::RefCell;

pub struct Context {
    module_builder: fmm::build::ModuleBuilder,
    types: FnvHashMap<String, mir::types::RecordBody>,
    fmm_types: RefCell<FnvHashMap<mir::types::Type, fmm::types::Type>>,
    type_information: mir::ir::TypeInformation,
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
            fmm_types: Default::default(),
            type_information: module.type_information().clone(),
            configuration,
        }
    }

    pub fn module_builder(&self) -> &fmm::build::ModuleBuilder {
        &self.module_builder
    }

    pub fn types(&self) -> &FnvHashMap<String, mir::types::RecordBody> {
        &self.types
    }

    pub fn type_information(&self) -> &mir::ir::TypeInformation {
        &self.type_information
    }

    pub fn fmm_types(&self) -> &RefCell<FnvHashMap<mir::types::Type, fmm::types::Type>> {
        &self.fmm_types
    }

    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }

    pub fn into_module_builder(self) -> fmm::build::ModuleBuilder {
        self.module_builder
    }
}
