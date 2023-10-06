use core::str;

const INVALID_CODE_POINT: ffi::Number = ffi::Number::new(f64::NAN);

#[ffi::bindgen]
fn _pen_core_character_from_code_point(code_point: ffi::Number) -> ffi::ByteString {
    char::from_u32(f64::from(code_point) as u32)
        .map(ffi::ByteString::from)
        .unwrap_or_default()
}

#[ffi::bindgen]
fn _pen_core_character_to_code_point(string: ffi::ByteString) -> ffi::Number {
    if let Ok(string) = str::from_utf8(string.as_slice()) {
        string
            .chars()
            .next()
            .map(|character| (character as u32 as f64).into())
            .unwrap_or(INVALID_CODE_POINT)
    } else {
        INVALID_CODE_POINT
    }
}
