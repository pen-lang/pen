#[ffi::bindgen]
fn _pen_core_view_has_prefix(
    string: ffi::ByteString,
    index: ffi::Number,
    prefix: ffi::ByteString,
) -> ffi::Boolean {
    let string = string.as_slice();
    let index = f64::from(index) as usize - 1;
    let prefix = prefix.as_slice();

    (&string[index..index + prefix.len()] == prefix).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_prefix() {
        assert!(bool::from(_pen_core_view_has_prefix(
            "foo bar baz".into(),
            5.0.into(),
            "bar".into()
        )));
    }

    #[test]
    fn fail_to_check_prefix() {
        assert!(!bool::from(_pen_core_view_has_prefix(
            "foo".into(),
            1.0.into(),
            "bar".into()
        )));
    }
}
