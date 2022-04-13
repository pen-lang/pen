use alloc::vec;
use core::str;

#[repr(C)]
struct FirstRest {
    ok: bool,
    first: ffi::Any,
    rest: ffi::Arc<ffi::List>,
}

extern "C" {
    fn _pen_core_first_rest(xs: ffi::Arc<ffi::List>) -> ffi::Arc<FirstRest>;
    fn _pen_core_to_string(xs: ffi::BoxAny) -> ffi::ByteString;
}

#[ffi::bindgen]
fn _pen_core_join_strings(
    mut list: ffi::Arc<ffi::List>,
    separator: ffi::ByteString,
) -> ffi::ByteString {
    let mut first = true;
    let mut string = vec![];

    loop {
        let first_rest = unsafe { _pen_core_first_rest(list.clone()) };

        if !first_rest.ok {
            return string.into();
        } else if !first {
            string.extend(separator.as_slice());
        }

        first = false;
        string.extend(unsafe { _pen_core_to_string(first_rest.first.clone().into()) }.as_slice());
        list = first_rest.rest.clone();
    }
}

#[ffi::bindgen]
fn _pen_core_utf8_length(string: ffi::ByteString) -> ffi::Number {
    if let Ok(string) = str::from_utf8(string.as_slice()) {
        string.chars().count() as f64
    } else {
        f64::NAN
    }
    .into()
}

#[ffi::bindgen]
fn _pen_core_utf8_slice(
    string: ffi::ByteString,
    start: ffi::Number,
    end: ffi::Number,
) -> ffi::ByteString {
    string.char_slice(start, end)
}

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
    let end = f64::from(end) as usize;

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
