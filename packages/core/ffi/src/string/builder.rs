use alloc::{sync, vec::Vec};

const INITIAL_STRING_BUILDER_CAPACITY: usize = 16;

#[repr(C)]
struct StringBuilder(ffi::Any);

#[ffi::any]
#[derive(Clone)]
struct StringBuilderInner {
    #[allow(clippy::box_collection)]
    strings: sync::Arc<Vec<ffi::ByteString>>,
}

impl StringBuilder {
    pub fn new() -> Self {
        Self(
            StringBuilderInner {
                strings: Vec::with_capacity(INITIAL_STRING_BUILDER_CAPACITY).into(),
            }
            .into(),
        )
    }
}

#[ffi::bindgen]
fn _pen_core_string_builder_create() -> StringBuilder {
    StringBuilder::new()
}

#[ffi::bindgen]
fn _pen_core_string_builder_append(
    builder: StringBuilder,
    string: ffi::ByteString,
) -> StringBuilder {
    let mut inner = StringBuilderInner::try_from(builder.0).unwrap();

    if let Some(strings) = sync::Arc::get_mut(&mut inner.strings) {
        strings.push(string);
    }

    StringBuilder(inner.into())
}

#[ffi::bindgen]
fn _pen_core_string_builder_build(
    builder: StringBuilder,
    separator: ffi::ByteString,
) -> ffi::ByteString {
    let inner: &StringBuilderInner = (&builder.0).try_into().unwrap();

    inner
        .strings
        .iter()
        .map(|string| string.as_slice())
        .collect::<Vec<_>>()
        .join(separator.as_slice())
        .into()
}
