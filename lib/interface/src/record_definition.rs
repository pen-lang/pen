use hir::types;
use position::Position;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RecordDefinition {
    name: String,
    original_name: String,
    fields: Vec<types::RecordField>,
    open: bool,
    public: bool,
    position: Position,
}

impl RecordDefinition {
    pub fn new(
        name: impl Into<String>,
        original_name: impl Into<String>,
        fields: Vec<types::RecordField>,
        open: bool,
        public: bool,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            original_name: original_name.into(),
            fields,
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

    pub fn fields(&self) -> &[types::RecordField] {
        &self.fields
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
