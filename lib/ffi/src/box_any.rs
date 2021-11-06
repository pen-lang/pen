use super::{any::Any, arc::Arc};
use std::ops::Deref;

#[repr(C)]
pub struct BoxAny {
    inner: Arc<BoxAnyInner>,
}

struct BoxAnyInner {
    value: Any,
}

impl BoxAny {
    pub fn new(value: Any) -> Self {
        Self {
            inner: BoxAnyInner { value }.into(),
        }
    }
}

impl Deref for BoxAny {
    type Target = Any;

    fn deref(&self) -> &Any {
        &self.inner.value
    }
}
