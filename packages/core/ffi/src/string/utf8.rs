use core::str;

#[ffi::bindgen]
fn _pen_core_utf8_contains(string: ffi::ByteString, pattern: ffi::ByteString) -> ffi::Boolean {
    if let Ok(string) = str::from_utf8(string.as_slice()) {
        if let Ok(pattern) = str::from_utf8(pattern.as_slice()) {
            string.contains(pattern)
        } else {
            false
        }
    } else {
        false
    }
    .into()
}

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

    if let Ok(string) = str::from_utf8(string.as_slice()) {
        if string.is_empty() || start >= string.chars().count() || end <= start {
            Default::default()
        } else {
            string[get_utf8_byte_index(string, start)..get_utf8_byte_index(string, end)].into()
        }
    } else {
        Default::default()
    }
}

fn get_utf8_byte_index(string: &str, index: usize) -> usize {
    string
        .char_indices()
        .nth(index)
        .map(|(index, _)| index)
        .unwrap_or_else(|| string.as_bytes().len())
}

#[ffi::bindgen]
fn _pen_core_utf8_trim(string: ffi::ByteString) -> ffi::ByteString {
    trim(string, |string| string.trim())
}

#[ffi::bindgen]
fn _pen_core_utf8_trim_end(string: ffi::ByteString) -> ffi::ByteString {
    trim(string, |string| string.trim_end())
}

#[ffi::bindgen]
fn _pen_core_utf8_trim_end_matches(
    string: ffi::ByteString,
    pattern: ffi::ByteString,
) -> ffi::ByteString {
    trim_matches(string, pattern, |string, pattern| {
        string.trim_end_matches(pattern)
    })
}

#[ffi::bindgen]
fn _pen_core_utf8_trim_start(string: ffi::ByteString) -> ffi::ByteString {
    trim(string, |string| string.trim_start())
}

#[ffi::bindgen]
fn _pen_core_utf8_trim_start_matches(
    string: ffi::ByteString,
    pattern: ffi::ByteString,
) -> ffi::ByteString {
    trim_matches(string, pattern, |string, pattern| {
        string.trim_start_matches(pattern)
    })
}

fn trim(
    original: ffi::ByteString,
    callback: for<'a, 'b> fn(&'a str) -> &'a str,
) -> ffi::ByteString {
    if let Ok(string) = str::from_utf8(original.as_slice()) {
        callback(string).into()
    } else {
        original
    }
}

fn trim_matches(
    original: ffi::ByteString,
    pattern: ffi::ByteString,
    callback: for<'a, 'b> fn(&'a str, &'b str) -> &'a str,
) -> ffi::ByteString {
    if let Ok(string) = str::from_utf8(original.as_slice()) {
        if let Ok(pattern) = str::from_utf8(pattern.as_slice()) {
            callback(string, pattern).into()
        } else {
            original
        }
    } else {
        original
    }
}
