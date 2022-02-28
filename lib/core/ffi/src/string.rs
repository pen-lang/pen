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
fn _pen_join_strings(mut list: ffi::Arc<ffi::List>, separator: ffi::ByteString) -> ffi::ByteString {
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
fn _pen_slice_string(
    string: ffi::ByteString,
    start: ffi::Number,
    end: ffi::Number,
) -> ffi::ByteString {
    string.char_slice(start, end)
}
