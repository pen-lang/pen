use std::rc::Rc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ByteString {
    value: Rc<[u8]>,
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
