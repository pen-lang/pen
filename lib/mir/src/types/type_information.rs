use super::Type;
use crate::types;
use fnv::FnvHashMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeInformation {
    types: Vec<types::Function>,
    information: FnvHashMap<Type, Vec<String>>,
}

impl TypeInformation {
    pub fn new(types: Vec<types::Function>, information: FnvHashMap<Type, Vec<String>>) -> Self {
        Self { types, information }
    }

    pub fn types(&self) -> &[types::Function] {
        &self.types
    }

    pub fn information(&self) -> &FnvHashMap<Type, Vec<String>> {
        &self.information
    }
}
