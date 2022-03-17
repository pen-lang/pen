use alloc::vec;

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
    string.as_slice()[f64::from(start) as usize - 1..f64::from(end) as usize].into()
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
}
