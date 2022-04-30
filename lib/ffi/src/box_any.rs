use super::{any::Any, arc::Arc};
use core::ops::Deref;

#[repr(C)]
#[derive(Clone)]
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

impl From<Any> for BoxAny {
    fn from(x: Any) -> Self {
        Self::new(x)
    }
}
