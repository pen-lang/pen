use std::str;

#[ffi::bindgen]
pub fn _pen_debug(message: ffi::ByteString) {
    eprintln!(
        "{}",
        str::from_utf8(message.as_slice()).unwrap_or("failed to decode debug message")
    );
}
