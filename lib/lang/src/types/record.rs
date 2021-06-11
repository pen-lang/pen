use super::record_element::RecordElement;
use crate::position::Position;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct Record {
    name: String,
    elements: Vec<RecordElement>,
    position: Position,
}

impl Record {
    pub fn new(name: impl Into<String>, elements: Vec<RecordElement>, position: Position) -> Self {
        Self {
            name: name.into(),
            elements,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn elements(&self) -> &[RecordElement] {
        &self.elements
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
