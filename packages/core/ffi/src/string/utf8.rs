use alloc::{string::String, vec::Vec};
use core::str;

#[ffi::bindgen]
fn _pen_core_utf8_characters(string: ffi::ByteString) -> ffi::List {
    if let Ok(string) = str::from_utf8(string.as_slice()) {
        string
            .chars()
            .map(|character| character.into())
            .collect::<Vec<ffi::ByteString>>()
            .into()
    } else {
        Default::default()
    }
}

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
fn _pen_core_utf8_find(string: ffi::ByteString, pattern: ffi::ByteString) -> ffi::Number {
    find(string, pattern)
        .map(|index| (index + 1) as f64)
        .unwrap_or(-1.0)
        .into()
}

fn find(string: ffi::ByteString, pattern: ffi::ByteString) -> Option<usize> {
    let string = str::from_utf8(string.as_slice()).ok()?;
    let pattern = str::from_utf8(pattern.as_slice()).ok()?;
    let index = string.find(pattern)?;

    Some(
        string
            .char_indices()
            .enumerate()
            .find(|(_, (byte_index, _))| byte_index == &index)?
            .0,
    )
}

#[ffi::bindgen]
fn _pen_core_utf8_starts_with(string: ffi::ByteString, pattern: ffi::ByteString) -> ffi::Boolean {
    if let Ok(string) = str::from_utf8(string.as_slice()) {
        if let Ok(pattern) = str::from_utf8(pattern.as_slice()) {
            string.starts_with(pattern)
        } else {
            false
        }
    } else {
        false
    }
    .into()
}

#[ffi::bindgen]
fn _pen_core_utf8_ends_with(string: ffi::ByteString, pattern: ffi::ByteString) -> ffi::Boolean {
    if let Ok(string) = str::from_utf8(string.as_slice()) {
        if let Ok(pattern) = str::from_utf8(pattern.as_slice()) {
            string.ends_with(pattern)
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

// TODO Split a string and collect sub-strings lazily.
#[ffi::bindgen]
fn _pen_core_utf8_split(original: ffi::ByteString, pattern: ffi::ByteString) -> ffi::List {
    if let Ok(string) = str::from_utf8(original.as_slice()) {
        if let Ok(pattern) = str::from_utf8(pattern.as_slice()) {
            string
                .split(pattern)
                .map(ffi::ByteString::from)
                .collect::<Vec<_>>()
                .into()
        } else {
            [original].into()
        }
    } else {
        [original].into()
    }
}

fn get_utf8_byte_index(string: &str, index: usize) -> usize {
    string
        .char_indices()
        .nth(index)
        .map(|(index, _)| index)
        .unwrap_or_else(|| string.len())
}

#[ffi::bindgen]
fn _pen_core_utf8_replace(
    original: ffi::ByteString,
    pattern: ffi::ByteString,
    replacement: ffi::ByteString,
) -> ffi::ByteString {
    if let Ok(string) = str::from_utf8(original.as_slice()) {
        if let Ok(pattern) = str::from_utf8(pattern.as_slice()) {
            if let Ok(replacement) = str::from_utf8(replacement.as_slice()) {
                string.replace(pattern, replacement).into()
            } else {
                original
            }
        } else {
            original
        }
    } else {
        original
    }
}

#[ffi::bindgen]
fn _pen_core_utf8_to_lowercase(string: ffi::ByteString) -> ffi::ByteString {
    convert_string(string, |string| string.to_lowercase())
}

#[ffi::bindgen]
fn _pen_core_utf8_to_uppercase(string: ffi::ByteString) -> ffi::ByteString {
    convert_string(string, |string| string.to_uppercase())
}

#[ffi::bindgen]
fn _pen_core_utf8_trim(string: ffi::ByteString) -> ffi::ByteString {
    convert_string(string, |string| string.trim().into())
}

#[ffi::bindgen]
fn _pen_core_utf8_trim_end(string: ffi::ByteString) -> ffi::ByteString {
    convert_string(string, |string| string.trim_end().into())
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
    convert_string(string, |string| string.trim_start().into())
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

fn convert_string(original: ffi::ByteString, callback: fn(&str) -> String) -> ffi::ByteString {
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
