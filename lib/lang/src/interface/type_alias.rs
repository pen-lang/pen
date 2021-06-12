use crate::types::Type;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct TypeAlias {
    name: String,
    type_: Type,
    public: bool,
}

impl TypeAlias {
    pub fn new(name: impl Into<String>, type_: impl Into<Type>, public: bool) -> Self {
        Self {
            name: name.into(),
            type_: type_.into(),
            public,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn is_public(&self) -> bool {
        self.public
    }
}
