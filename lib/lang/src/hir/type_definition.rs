use crate::{position::Position, types};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub struct TypeDefinition {
    name: String,
    elements: Vec<types::RecordElement>,
    open: bool,
    public: bool,
    external: bool,
    position: Position,
}

impl TypeDefinition {
    pub fn new(
        name: impl Into<String>,
        elements: Vec<types::RecordElement>,
        open: bool,
        public: bool,
        external: bool,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            elements,
            open,
            public,
            external,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn elements(&self) -> &[types::RecordElement] {
        &self.elements
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

    pub fn position(&self) -> &Position {
        &self.position
    }
}
