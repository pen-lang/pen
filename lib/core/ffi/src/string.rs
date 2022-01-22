// TODO Move to a `pen-ffi` crate
#[repr(C)]
struct List {
    node: ffi::Arc<ffi::Closure>,
}

#[repr(C)]
struct FirstRest {
    ok: bool,
    first: ffi::Any,
    rest: ffi::Arc<List>,
}

extern "C" {
    fn _pen_core_first_rest(xs: ffi::Arc<List>) -> ffi::Arc<FirstRest>;
    fn _pen_core_to_string(xs: ffi::Any) -> ffi::ByteString;
}

#[ffi::bindgen]
fn _pen_join_strings(mut list: ffi::Arc<List>, separator: ffi::ByteString) -> ffi::ByteString {
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
        string.extend(unsafe { _pen_core_to_string(first_rest.first.clone()) }.as_slice());
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
