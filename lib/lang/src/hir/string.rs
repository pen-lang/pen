use crate::position::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ByteString {
    // TODO Change it to Vec<u8>.
    value: String,
    position: Position,
}

impl ByteString {
    pub fn new(value: impl Into<String>, position: Position) -> Self {
        Self {
            value: value.into(),
            position,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
