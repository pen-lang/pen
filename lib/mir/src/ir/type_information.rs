use crate::{types, types::Type};
use fnv::FnvHashMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeInformation {
    types: Vec<types::Function>,
    information: FnvHashMap<Type, Vec<String>>,
    fallback: Vec<String>,
}

impl TypeInformation {
    pub fn new(
        types: Vec<types::Function>,
        information: FnvHashMap<Type, Vec<String>>,
        fallback: Vec<String>,
    ) -> Self {
        Self {
            types,
            information,
            fallback,
        }
    }

    pub fn types(&self) -> &[types::Function] {
        &self.types
    }

    pub fn information(&self) -> &FnvHashMap<Type, Vec<String>> {
        &self.information
    }

    pub fn fallback(&self) -> &[String] {
        &self.fallback
    }
}
