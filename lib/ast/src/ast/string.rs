use position::Position;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ByteString {
    value: String, // UTF-8 representation of byte string
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
