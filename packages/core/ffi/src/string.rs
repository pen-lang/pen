use alloc::boxed::Box;
use alloc::vec::Vec;
use core::str;
use futures::stream::StreamExt;

#[ffi::bindgen]
async fn _pen_core_string_join(
    list: ffi::Arc<ffi::List>,
    separator: ffi::ByteString,
) -> ffi::ByteString {
    let elements = ffi::future::stream::from_list(list);

    futures::pin_mut!(elements);

    let mut strings = Vec::new();

    while let Some(element) = elements.next().await {
        strings.push(ffi::BoxAny::from(element).to_string().await);
    }

    strings
        .iter()
        .map(|string| string.as_slice())
        .collect::<Vec<_>>()
        .join(separator.as_slice())
        .into()
}

#[ffi::bindgen]
fn _pen_core_string_has_prefix(string: ffi::ByteString, prefix: ffi::ByteString) -> ffi::Boolean {
    let string = string.as_slice();
    let prefix = prefix.as_slice();

    (string.len() >= prefix.len() && &string[..prefix.len()] == prefix).into()
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

#[ffi::bindgen]
fn _pen_core_byte_length(string: ffi::ByteString) -> ffi::Number {
    (string.as_slice().len() as f64).into()
}

#[ffi::bindgen]
fn _pen_core_byte_slice(
    string: ffi::ByteString,
    start: ffi::Number,
    end: ffi::Number,
) -> ffi::ByteString {
    let start = f64::from(start) as usize;
    let end = f64::from(end).min(string.as_slice().len() as f64) as usize;

    if start > end || start > string.as_slice().len() || end == 0 {
        "".into()
    } else {
        string.as_slice()[start - 1..end].into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_bytes() {
        assert_eq!(
            _pen_core_byte_slice("hello".into(), 2.0.into(), 4.0.into()),
            "ell".into()
        );
    }

    #[test]
    fn slice_bytes_with_too_large_index() {
        assert_eq!(
            _pen_core_byte_slice("hello".into(), 6.0.into(), 6.0.into()),
            "".into()
        );
    }

    #[test]
    fn slice_bytes_with_too_small_index() {
        assert_eq!(
            _pen_core_byte_slice("hello".into(), 0.0.into(), 0.0.into()),
            "".into()
        );
    }

    #[test]
    fn slice_bytes_with_negative_index() {
        assert_eq!(
            _pen_core_byte_slice("hello".into(), (-1.0).into(), (-1.0).into()),
            "".into()
        );
    }
}
