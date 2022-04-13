#![no_std]

extern crate alloc;

use core::str;

#[ffi::bindgen]
fn _pen_json_parse_number(string: ffi::ByteString) -> ffi::Number {
    str::from_utf8(string.as_slice())
        .unwrap_or("")
        .parse::<f64>()
        .unwrap_or(f64::NAN)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_integer() {
        assert_eq!(_pen_json_parse_number("42".into()), 42.0.into());
    }

    #[test]
    fn parse_number() {
        assert_eq!(_pen_json_parse_number("42.0".into()), 42.0.into());
    }
}
