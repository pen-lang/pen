use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ByteString {
    value: Arc<[u8]>,
}

impl ByteString {
    pub fn new(value: impl Into<Vec<u8>>) -> Self {
        Self {
            value: value.into().into(),
        }
    }

    pub fn value(&self) -> &[u8] {
        &self.value
    }
}
