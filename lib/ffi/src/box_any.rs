use super::{any::Any, arc::Arc};
use core::ops::Deref;

#[repr(C)]
#[derive(Clone)]
pub struct BoxAny(Arc<Any>);

impl BoxAny {
    pub fn new(value: impl Into<Any>) -> Self {
        Self(Arc::new(value.into()))
    }
}

impl Deref for BoxAny {
    type Target = Any;

    fn deref(&self) -> &Any {
        &self.0
    }
}

impl From<Any> for BoxAny {
    fn from(x: Any) -> Self {
        Self::new(x)
    }
}

impl From<BoxAny> for Any {
    fn from(x: BoxAny) -> Self {
        x.0.deref().clone()
    }
}
