use super::{arc::ArcBuffer, number::Number};
use crate::type_information;
use std::{cmp::max, str::from_utf8_unchecked};

#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct ByteString {
    buffer: ArcBuffer,
}

impl ByteString {
    pub fn new(buffer: ArcBuffer) -> Self {
        Self { buffer }
    }

    pub fn empty() -> Self {
        Self {
            buffer: ArcBuffer::new(0),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.buffer.as_slice()
    }

    pub fn len(&self) -> usize {
        self.buffer.as_slice().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn join(&self, other: &Self) -> Self {
        let mut buffer = ArcBuffer::new(self.len() + other.len());

        buffer.as_slice_mut()[..self.len()].copy_from_slice(self.as_slice());
        buffer.as_slice_mut()[self.len()..].copy_from_slice(other.as_slice());

        Self { buffer }
    }

    // Indices are inclusive and start from 1.
    pub fn slice(&self, start: Number, end: Number) -> ByteString {
        let start = f64::from(start);
        let end = f64::from(end);

        // TODO Allow infinite ranges
        if !start.is_finite() || !end.is_finite() {
            return Self::empty();
        }

        let start = max(start as isize - 1, 0) as usize;
        let end = max(end as isize, 0) as usize;

        let string = unsafe { from_utf8_unchecked(self.as_slice()) };

        if string.is_empty() || start >= string.chars().count() || end <= start {
            Self::empty()
        } else {
            string[Self::get_byte_index(string, start)..Self::get_byte_index(string, end)].into()
        }
    }

    fn get_byte_index(string: &str, index: usize) -> usize {
        string
            .char_indices()
            .nth(index)
            .map(|(index, _)| index)
            .unwrap_or_else(|| string.as_bytes().len())
    }
}

impl PartialEq for ByteString {
    fn eq(&self, other: &ByteString) -> bool {
        self.as_slice() == other.as_slice()
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

impl From<Vec<u8>> for ByteString {
    fn from(vec: Vec<u8>) -> Self {
        vec.as_slice().into()
    }
}

type_information!(byte_string, crate::string::ByteString);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join() {
        assert_eq!(
            ByteString::from("foo").join(&ByteString::from("bar")),
            ByteString::from("foobar")
        );
    }

    #[test]
    fn join_empty() {
        assert_eq!(
            ByteString::from("").join(&ByteString::from("")),
            ByteString::from("")
        );
    }

    #[test]
    fn slice_with_ascii() {
        assert_eq!(
            ByteString::from("abc").slice(2.0.into(), 2.0.into()),
            ByteString::from("b")
        );
    }

    #[test]
    fn slice_with_negative_index() {
        assert_eq!(
            ByteString::from("abc").slice((-1.0).into(), 3.0.into()),
            ByteString::from("abc")
        );
    }

    #[test]
    fn slice_into_whole() {
        assert_eq!(
            ByteString::from("abc").slice(1.0.into(), 3.0.into()),
            ByteString::from("abc")
        );
    }

    #[test]
    fn slice_into_empty() {
        assert_eq!(
            ByteString::from("abc").slice(4.0.into(), 4.0.into()),
            ByteString::from("")
        );
    }

    #[test]
    fn slice_with_emojis() {
        assert_eq!(
            ByteString::from("😀😉😂").slice(2.0.into(), 2.0.into()),
            ByteString::from("😉")
        );
    }

    #[test]
    fn slice_last_with_emojis() {
        assert_eq!(
            ByteString::from("😀😉😂").slice(3.0.into(), 3.0.into()),
            ByteString::from("😂")
        );
    }
}
