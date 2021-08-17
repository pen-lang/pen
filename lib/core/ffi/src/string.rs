#[no_mangle]
fn _pen_join_strings(one: ffi::ByteString, other: ffi::ByteString) -> ffi::ByteString {
    one.join(&other)
}

#[no_mangle]
fn _pen_slice_string(
    string: ffi::ByteString,
    start: ffi::Number,
    end: ffi::Number,
) -> ffi::ByteString {
    string.slice(start, end)
}
