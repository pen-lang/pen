use crate::{Any, Arc};

#[repr(C)]
#[derive(Clone, Default)]
pub struct Error {
    source: Any,
}

impl Error {
    pub fn new(source: impl Into<Any>) -> Arc<Self> {
        Self {
            source: source.into(),
        }
        .into()
    }
}

#[cfg(feature = "std")]
impl<T: std::error::Error> From<T> for Arc<Error> {
    fn from(error: T) -> Self {
        use alloc::string::ToString;

        Error::new(crate::ByteString::from(error.to_string())).into()
    }
}
