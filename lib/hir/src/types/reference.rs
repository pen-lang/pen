use position::Position;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Reference(Arc<ReferenceInner>);

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
struct ReferenceInner {
    name: String,
    position: Position,
}

impl Reference {
    pub fn new(name: impl Into<String>, position: Position) -> Self {
        Self(
            ReferenceInner {
                name: name.into(),
                position,
            }
            .into(),
        )
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn position(&self) -> &Position {
        &self.0.position
    }

    pub fn set_position(&self, position: Position) -> Self {
        Self(
            ReferenceInner {
                name: self.0.name.clone(),
                position,
            }
            .into(),
        )
    }
}
