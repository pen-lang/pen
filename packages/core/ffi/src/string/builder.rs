use alloc::{boxed::Box, vec::Vec};

const INITIAL_STRING_BUILDER_CAPACITY: usize = 16;

#[repr(C)]
struct StringBuilder(ffi::Arc<ffi::Any>);

#[ffi::any]
#[derive(Clone)]
struct StringBuilderInner {
    #[allow(clippy::box_collection)]
    strings: Box<Vec<ffi::ByteString>>,
}

impl StringBuilder {
    pub fn new() -> Self {
        Self(ffi::Arc::new(
            StringBuilderInner {
                strings: Vec::with_capacity(INITIAL_STRING_BUILDER_CAPACITY).into(),
            }
            .into(),
        ))
    }
}

#[ffi::bindgen]
fn _pen_core_string_builder_create() -> StringBuilder {
    StringBuilder::new()
}

#[ffi::bindgen]
fn _pen_core_string_builder_append(
    mut builder: StringBuilder,
    string: ffi::ByteString,
) -> StringBuilder {
    if let Some(builder) = ffi::Arc::get_mut(&mut builder.0) {
        let inner: &mut StringBuilderInner = builder.try_into().unwrap();

        inner.strings.push(string);
    }

    builder
}

#[ffi::bindgen]
fn _pen_core_string_builder_build(
    builder: StringBuilder,
    separator: ffi::ByteString,
) -> ffi::ByteString {
    let inner: &StringBuilderInner = (&*builder.0).try_into().unwrap();

    inner
        .strings
        .iter()
        .map(|string| string.as_slice())
        .collect::<Vec<_>>()
        .join(separator.as_slice())
        .into()
}
