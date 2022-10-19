use position::Position;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct Record(Arc<RecordInner>);

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
struct RecordInner {
    name: String,
    original_name: String,
    position: Position,
}

impl Record {
    pub fn new(
        name: impl Into<String>,
        original_name: impl Into<String>,
        position: Position,
    ) -> Self {
        Self(
            RecordInner {
                name: name.into().into(),
                original_name: original_name.into().into(),
                position,
            }
            .into(),
        )
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn original_name(&self) -> &str {
        &self.0.original_name
    }

    pub fn position(&self) -> &Position {
        &self.0.position
    }

    pub fn set_position(&self, position: Position) -> Self {
        Self(
            RecordInner {
                name: self.0.name.clone(),
                original_name: self.0.original_name.clone(),
                position,
            }
            .into(),
        )
    }
}
