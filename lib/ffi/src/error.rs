use crate::{Any, Arc, ByteString};
use alloc::string::ToString;

#[repr(C)]
#[derive(Clone, Default)]
pub struct Error {
    source: Any,
}

impl Error {
    pub fn new(source: Any) -> Arc<Self> {
        Arc::new(Self { source })
    }
}

impl<T: ToString> From<T> for Arc<Error> {
    fn from(x: T) -> Self {
        Error::new(ByteString::from(x.to_string()).into()).into()
    }
}
