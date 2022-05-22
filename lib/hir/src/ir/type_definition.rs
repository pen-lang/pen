use crate::types;
use position::Position;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct TypeDefinition {
    name: String,
    original_name: String,
    fields: Vec<types::RecordField>,
    open: bool,
    public: bool,
    external: bool,
    position: Position,
}

impl TypeDefinition {
    pub fn new(
        name: impl Into<String>,
        original_name: impl Into<String>,
        fields: Vec<types::RecordField>,
        open: bool,
        public: bool,
        external: bool,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            original_name: original_name.into(),
            fields,
            open,
            public,
            external,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn original_name(&self) -> &str {
        &self.original_name
    }

    pub fn fields(&self) -> &[types::RecordField] {
        &self.fields
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn is_public(&self) -> bool {
        self.public
    }

    pub fn is_external(&self) -> bool {
        self.external
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
