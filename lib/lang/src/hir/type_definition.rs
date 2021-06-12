use crate::types;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct TypeDefinition {
    name: String,
    type_: types::Record,
    open: bool,
    public: bool,
    external: bool,
}

impl TypeDefinition {
    pub fn new(
        name: impl Into<String>,
        type_: types::Record,
        public: bool,
        open: bool,
        external: bool,
    ) -> Self {
        Self {
            name: name.into(),
            type_,
            open,
            public,
            external,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> &types::Record {
        &self.type_
    }

    pub fn is_open(&self) -> bool {
        self.public
    }

    pub fn is_public(&self) -> bool {
        self.public
    }

    pub fn is_external(&self) -> bool {
        self.public
    }
}
