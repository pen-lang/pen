use hir::types::Type;
use position::Position;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct TypeAlias {
    name: String,
    original_name: String,
    type_: Type,
    public: bool,
    position: Position,
}

impl TypeAlias {
    pub fn new(
        name: impl Into<String>,
        original_name: impl Into<String>,
        type_: impl Into<Type>,
        public: bool,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            original_name: original_name.into(),
            type_: type_.into(),
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

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn is_public(&self) -> bool {
        self.public
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
