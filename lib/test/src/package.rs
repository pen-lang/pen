use crate::Module;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Package {
    modules: BTreeMap<String, Module>,
}

impl Package {
    pub fn new(modules: BTreeMap<String, Module>) -> Self {
        Self { modules }
    }

    pub fn modules(&self) -> &BTreeMap<String, Module> {
        &self.modules
    }
}
