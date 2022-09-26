use crate::types;
use fnv::FnvHashMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeInformation {
    types: Vec<types::Function>,
    information: FnvHashMap<String, Vec<String>>,
}

impl TypeInformation {
    pub fn new(types: Vec<types::Function>, information: FnvHashMap<String, Vec<String>>) -> Self {
        Self { types, information }
    }

    pub fn types(&self) -> &[types::Function] {
        &self.types
    }

    pub fn information(&self) -> &FnvHashMap<String, Vec<String>> {
        &self.information
    }
}
