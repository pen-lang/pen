#[ffi::bindgen]
fn _pen_core_byte_length(string: ffi::ByteString) -> ffi::Number {
    (string.as_slice().len() as f64).into()
}

#[ffi::bindgen]
fn _pen_core_byte_slice(
    string: ffi::ByteString,
    start: ffi::Number,
    end: ffi::Number,
) -> ffi::ByteString {
    let start = f64::from(start) as usize;
    let end = f64::from(end).min(string.as_slice().len() as f64) as usize;

    if start > end || start > string.as_slice().len() || end == 0 {
        "".into()
    } else {
        string.as_slice()[start - 1..end].into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_bytes() {
        assert_eq!(
            _pen_core_byte_slice("hello".into(), 2.0.into(), 4.0.into()),
            "ell".into()
        );
    }

    #[test]
    fn slice_bytes_with_too_large_index() {
        assert_eq!(
            _pen_core_byte_slice("hello".into(), 6.0.into(), 6.0.into()),
            "".into()
        );
    }

    #[test]
    fn slice_bytes_with_too_small_index() {
        assert_eq!(
            _pen_core_byte_slice("hello".into(), 0.0.into(), 0.0.into()),
            "".into()
        );
    }

    #[test]
    fn slice_bytes_with_negative_index() {
        assert_eq!(
            _pen_core_byte_slice("hello".into(), (-1.0).into(), (-1.0).into()),
            "".into()
        );
    }
}
