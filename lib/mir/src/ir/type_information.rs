use crate::types::Type;
use fnv::FnvHashMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeInformation {
    information: FnvHashMap<Type, String>,
    fallback: String,
}

impl TypeInformation {
    pub fn new(information: FnvHashMap<Type, String>, fallback: String) -> Self {
        Self {
            information,
            fallback,
        }
    }

    pub fn information(&self) -> &FnvHashMap<Type, String> {
        &self.information
    }

    pub fn fallback(&self) -> &str {
        &self.fallback
    }
}
