use fnv::FnvHashMap;

pub struct Context {
    module_builder: fmm::build::ModuleBuilder,
    types: FnvHashMap<String, mir::types::RecordBody>,
}

impl Context {
    pub fn new(module: &mir::ir::Module) -> Self {
        Self {
            module_builder: fmm::build::ModuleBuilder::new(),
            types: module
                .type_definitions()
                .iter()
                .map(|definition| (definition.name().into(), definition.type_().clone()))
                .collect(),
        }
    }

    pub fn module_builder(&self) -> &fmm::build::ModuleBuilder {
        &self.module_builder
    }

    pub fn types(&self) -> &FnvHashMap<String, mir::types::RecordBody> {
        &self.types
    }
}
