#[derive(Clone, Debug, PartialEq)]
pub struct ByteString {
    value: Vec<u8>,
}

impl ByteString {
    pub fn new(value: impl Into<Vec<u8>>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn value(&self) -> &[u8] {
        &self.value
    }
}
