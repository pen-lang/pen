#![no_std]

extern crate alloc;

use alloc::string::ToString;
use core::str;

#[ffi::bindgen]
fn _pen_json_decode_number(string: ffi::ByteString) -> ffi::Number {
    str::from_utf8(string.as_slice())
        .unwrap_or("")
        .decode::<f64>()
        .unwrap_or(f64::NAN)
        .into()
}

#[ffi::bindgen]
fn _pen_json_encode_number(number: ffi::Number) -> ffi::ByteString {
    f64::from(number).to_string().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_integer() {
        assert_eq!(_pen_json_decode_number("42".into()), 42.0.into());
    }

    #[test]
    fn decode_number() {
        assert_eq!(_pen_json_decode_number("42.0".into()), 42.0.into());
    }
}
