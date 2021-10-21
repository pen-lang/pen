use std::collections::BTreeSet;

pub struct ImportedModule {
    interface: interface::Module,
    prefix: String,
    unqualified_names: BTreeSet<String>,
}

impl ImportedModule {
    pub fn new(
        interface: interface::Module,
        prefix: impl Into<String>,
        unqualified_names: BTreeSet<String>,
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

    pub fn unqualified_names(&self) -> &BTreeSet<String> {
        &self.unqualified_names
    }
}
