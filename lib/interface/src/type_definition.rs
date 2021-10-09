use hir::types;
use position::Position;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct TypeDefinition {
    name: String,
    original_name: String,
    elements: Vec<types::RecordField>,
    open: bool,
    public: bool,
    position: Position,
}

impl TypeDefinition {
    pub fn new(
        name: impl Into<String>,
        original_name: impl Into<String>,
        elements: Vec<types::RecordField>,
        open: bool,
        public: bool,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            original_name: original_name.into(),
            elements,
            open,
            public,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn original_name(&self) -> &str {
        &self.original_name
    }

    pub fn elements(&self) -> &[types::RecordField] {
        &self.elements
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn is_public(&self) -> bool {
        self.public
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
