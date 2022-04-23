use super::{any::Any, arc::Arc};
use crate::{call_function, import, ByteString};
use core::ops::Deref;

#[repr(C)]
#[derive(Clone)]
pub struct BoxAny {
    inner: Arc<BoxAnyInner>,
}

struct BoxAnyInner {
    value: Any,
}

extern "C" {
    import!(_pen_ffi_any_to_string, fn(any: BoxAny) -> ByteString);
}

impl BoxAny {
    pub fn new(value: Any) -> Self {
        Self {
            inner: BoxAnyInner { value }.into(),
        }
    }

    pub async fn to_string(&self) -> ByteString {
        call_function!(
            fn(BoxAny) -> ByteString,
            _pen_ffi_any_to_string,
            self.clone()
        )
        .await
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
