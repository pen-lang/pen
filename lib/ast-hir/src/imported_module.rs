pub mod validation;

use fnv::FnvHashMap;
use position::Position;

pub struct ImportedModule {
    interface: interface::Module,
    prefix: String,
    unqualified_names: FnvHashMap<String, Position>,
}

impl ImportedModule {
    pub fn new(
        interface: interface::Module,
        prefix: impl Into<String>,
        unqualified_names: FnvHashMap<String, Position>,
    ) -> Self {
        Self {
            interface,
            prefix: prefix.into(),
            unqualified_names,
        }
    }

    pub fn interface(&self) -> &interface::Module {
        &self.interface
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn unqualified_names(&self) -> &FnvHashMap<String, Position> {
        &self.unqualified_names
    }
}
