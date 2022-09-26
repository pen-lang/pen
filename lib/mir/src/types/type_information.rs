use super::Type;
use fnv::FnvHashMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeInformation {
    types: Vec<Type>,
    information: FnvHashMap<String, Type>,
}

impl TypeInformation {
    pub fn new(types: Vec<Type>, information: FnvHashMap<String, Type>) -> Self {
        Self { types, information }
    }

    pub fn types(&self) -> &[Type] {
        &self.types
    }

    pub fn information(&self) -> &FnvHashMap<String, Type> {
        &self.information
    }
}
