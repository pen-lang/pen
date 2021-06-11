use crate::types;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct TypeDefinition {
    name: String,
    type_: types::Record,
    public: bool,
}

impl TypeDefinition {
    pub fn new(name: impl Into<String>, type_: types::Record, public: bool) -> Self {
        Self {
            name: name.into(),
            type_,
            public,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> &types::Record {
        &self.type_
    }

    pub fn is_public(&self) -> bool {
        self.public
    }
}
