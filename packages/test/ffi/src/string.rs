mod builder;
mod byte;
mod utf8;

#[ffi::bindgen]
fn _pen_core_string_starts_with(string: ffi::ByteString, prefix: ffi::ByteString) -> ffi::Boolean {
    let string = string.as_slice();
    let prefix = prefix.as_slice();

    (string.len() >= prefix.len() && &string[..prefix.len()] == prefix).into()
}
