use alloc::{boxed::Box, vec::Vec};

const INITIAL_STRING_BUILDER_CAPACITY: usize = 16;

#[repr(C)]
struct StringBuilder {
    inner: ffi::Any,
}

#[ffi::any]
#[derive(Clone)]
struct StringBuilderInner {
    #[allow(clippy::box_collection)]
    strings: Box<Vec<ffi::ByteString>>,
}

#[ffi::bindgen]
fn _pen_core_string_builder_create() -> ffi::Arc<StringBuilder> {
    StringBuilder {
        inner: StringBuilderInner {
            strings: Vec::with_capacity(INITIAL_STRING_BUILDER_CAPACITY).into(),
        }
        .into(),
    }
    .into()
}

#[ffi::bindgen]
fn _pen_core_string_builder_append(
    mut builder: ffi::Arc<StringBuilder>,
    string: ffi::ByteString,
) -> ffi::Arc<StringBuilder> {
    if let Some(builder) = ffi::Arc::get_mut(&mut builder) {
        let inner: &mut StringBuilderInner = (&mut builder.inner).try_into().unwrap();

        inner.strings.push(string);
    }

    builder
}

#[ffi::bindgen]
fn _pen_core_string_builder_build(
    builder: ffi::Arc<StringBuilder>,
    separator: ffi::ByteString,
) -> ffi::ByteString {
    let inner: &StringBuilderInner = (&builder.inner).try_into().unwrap();

    inner
        .strings
        .iter()
        .map(|string| string.as_slice())
        .collect::<Vec<_>>()
        .join(separator.as_slice())
        .into()
}
