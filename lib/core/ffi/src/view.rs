#[ffi::bindgen]
fn _pen_core_view_has_prefix(
    string: ffi::ByteString,
    start: ffi::Number,
    end: ffi::Number,
    prefix: ffi::ByteString,
) -> ffi::Boolean {
    let string = string.as_slice();
    let start = f64::from(start) as usize - 1;
    let prefix = prefix.as_slice();

    (&string[start..(start + prefix.len()).min(f64::from(end) as usize)] == prefix).into()
}

#[ffi::bindgen]
fn _pen_core_view_to_string(
    string: ffi::ByteString,
    start: ffi::Number,
    end: ffi::Number,
) -> ffi::ByteString {
    string.as_slice()[f64::from(start) as usize - 1..f64::from(end) as usize].into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_prefix() {
        assert!(bool::from(_pen_core_view_has_prefix(
            "foo bar baz".into(),
            5.0.into(),
            8.0.into(),
            "bar".into()
        )));
    }

    #[test]
    fn fail_to_check_prefix() {
        assert!(!bool::from(_pen_core_view_has_prefix(
            "foo".into(),
            1.0.into(),
            3.0.into(),
            "bar".into()
        )));
    }

    #[test]
    fn to_string() {
        assert_eq!(
            _pen_core_view_to_string("foo bar baz".into(), 5.0.into(), 7.0.into()),
            "bar".into()
        );
    }
}
