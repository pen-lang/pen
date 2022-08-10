#![no_std]

use core::str;

#[ffi::bindgen]
fn _pen_html_encode_text(string: ffi::ByteString) -> ffi::ByteString {
    html_escape::encode_text(str::from_utf8(string.as_slice()).unwrap_or_default())
        .as_bytes()
        .into()
}

#[ffi::bindgen]
fn _pen_html_encode_attribute(string: ffi::ByteString) -> ffi::ByteString {
    html_escape::encode_quoted_attribute(str::from_utf8(string.as_slice()).unwrap_or_default())
        .as_bytes()
        .into()
}
