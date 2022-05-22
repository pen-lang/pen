use position::Position;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ByteString {
    value: Vec<u8>,
    position: Position,
}

impl ByteString {
    pub fn new(value: impl Into<Vec<u8>>, position: Position) -> Self {
        Self {
            value: value.into(),
            position,
        }
    }

    pub fn value(&self) -> &[u8] {
        &self.value
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
