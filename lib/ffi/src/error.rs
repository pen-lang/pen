use crate::{Any, Arc};

#[pen_ffi_macro::into_any(crate = "crate", into_fn = "pen_ffi_error_to_any")]
#[repr(C)]
#[derive(Clone)]
pub struct Error(Arc<ErrorInner>);

#[repr(C)]
struct ErrorInner {
    source: Any,
}

impl Error {
    pub fn new(source: impl Into<Any>) -> Self {
        Self(
            ErrorInner {
                source: source.into(),
            }
            .into(),
        )
    }
}

#[cfg(feature = "std")]
impl<T: std::error::Error> From<T> for Error {
    fn from(error: T) -> Self {
        use alloc::string::ToString;

        Error::new(crate::ByteString::from(error.to_string()))
    }
}
