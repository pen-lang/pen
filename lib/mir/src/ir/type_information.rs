use crate::{types, types::Type};
use fnv::FnvHashMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeInformation {
    information: FnvHashMap<Type, Vec<String>>,
    fallback: Vec<String>,
}

impl TypeInformation {
    pub fn new(information: FnvHashMap<Type, Vec<String>>, fallback: Vec<String>) -> Self {
        Self {
            information,
            fallback,
        }
    }

    pub fn information(&self) -> &FnvHashMap<Type, Vec<String>> {
        &self.information
    }

    pub fn fallback(&self) -> &[String] {
        &self.fallback
    }
}
