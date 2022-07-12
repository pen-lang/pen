use core::str;

#[ffi::bindgen]
fn _pen_core_utf8_length(string: ffi::ByteString) -> ffi::Number {
    if let Ok(string) = str::from_utf8(string.as_slice()) {
        string.chars().count() as f64
    } else {
        f64::NAN
    }
    .into()
}

#[ffi::bindgen]
fn _pen_core_utf8_slice(
    string: ffi::ByteString,
    start: ffi::Number,
    end: ffi::Number,
) -> ffi::ByteString {
    let start = (f64::from(start) - 1.0).max(0.0) as usize;
    let end = f64::from(end).min(usize::MAX as f64) as usize;

    let string = if let Ok(string) = str::from_utf8(string.as_slice()) {
        string
    } else {
        return ffi::ByteString::default();
    };

    if string.is_empty() || start >= string.chars().count() || end <= start {
        ffi::ByteString::default()
    } else {
        string[get_utf8_byte_index(string, start)..get_utf8_byte_index(string, end)].into()
    }
}

fn get_utf8_byte_index(string: &str, index: usize) -> usize {
    string
        .char_indices()
        .nth(index)
        .map(|(index, _)| index)
        .unwrap_or_else(|| string.as_bytes().len())
}
