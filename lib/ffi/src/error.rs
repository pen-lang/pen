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
