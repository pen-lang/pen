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
impl From<alloc::boxed::Box<dyn std::error::Error>> for Arc<Error> {
    fn from(error: alloc::boxed::Box<dyn std::error::Error>) -> Self {
        use alloc::string::ToString;

        Error::new(crate::ByteString::from(error.to_string())).into()
    }
}
