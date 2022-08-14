use super::arc::ArcBuffer;
use alloc::{string::String, vec::Vec};
use core::hash::{Hash, Hasher};

#[pen_ffi_macro::into_any(crate = "crate", fn = "pen_ffi_string_to_any")]
#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct ByteString {
    buffer: ArcBuffer,
}

impl ByteString {
    pub fn as_slice(&self) -> &[u8] {
        self.buffer.as_slice()
    }
}

impl PartialEq for ByteString {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for ByteString {}

impl Hash for ByteString {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.as_slice().hash(hasher)
    }
}

impl From<char> for ByteString {
    fn from(character: char) -> Self {
        String::from(character).into()
    }
}

impl From<&[u8]> for ByteString {
    fn from(bytes: &[u8]) -> Self {
        Self {
            buffer: bytes.into(),
        }
    }
}

impl From<&str> for ByteString {
    fn from(string: &str) -> Self {
        string.as_bytes().into()
    }
}

impl From<String> for ByteString {
    fn from(string: String) -> Self {
        string.as_str().into()
    }
}

// TODO Use Vec::into_raw_parts().
impl From<Vec<u8>> for ByteString {
    fn from(vec: Vec<u8>) -> Self {
        vec.as_slice().into()
    }
}
