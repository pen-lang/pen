use crate::{Any, Arc};

#[repr(C)]
#[derive(Clone)]
pub struct Error {
    inner: Arc<ErrorInner>,
}

#[repr(C)]
struct ErrorInner {
    source: Any,
}

impl Error {
    pub fn new(source: impl Into<Any>) -> Self {
        Self {
            inner: ErrorInner {
                source: source.into(),
            }
            .into(),
        }
    }
}

#[cfg(feature = "std")]
impl<T: std::error::Error> From<T> for Arc<Error> {
    fn from(error: T) -> Self {
        use alloc::string::ToString;

        Error::new(crate::ByteString::from(error.to_string())).into()
    }
}
