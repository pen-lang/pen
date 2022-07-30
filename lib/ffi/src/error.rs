use crate::{Any, Arc};

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
