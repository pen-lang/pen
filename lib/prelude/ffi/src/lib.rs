#[no_mangle]
fn _pen_equal_strings(one: ffi::ByteString, other: ffi::ByteString) -> ffi::Boolean {
    (one.as_slice() == other.as_slice()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_empty_strings() {
        let string = ffi::ByteString::empty();

        assert_eq!(_pen_equal_strings(string.clone(), string), true.into());
    }

    #[test]
    fn equal_one_byte_strings() {
        let string = ffi::ByteString::from(vec![0u8]);

        assert_eq!(_pen_equal_strings(string.clone(), string), true.into());
    }

    #[test]
    fn not_equal_one_byte_strings() {
        let one = ffi::ByteString::empty();
        let other = vec![0u8].into();

        assert_eq!(_pen_equal_strings(one, other), false.into());
    }

    #[test]
    fn equal_text_strings() {
        const TEXT: &[u8] = "hello".as_bytes();

        let string = ffi::ByteString::from(TEXT);

        assert_eq!(_pen_equal_strings(string.clone(), string), true.into());
    }

    #[test]
    fn not_equal_text_strings() {
        const TEXT: &[u8] = "hello".as_bytes();
        const OTHER_TEXT: &[u8] = "hell0".as_bytes();

        assert_eq!(
            _pen_equal_strings(TEXT.into(), OTHER_TEXT.into(),),
            false.into()
        );
    }
}
