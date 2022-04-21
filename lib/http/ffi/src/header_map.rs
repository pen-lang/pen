#[ffi::any]
#[repr(C)]
#[derive(Clone)]
pub struct HeaderMap {
    headers: ffi::Any,
}

extern "C" {
    fn _pen_http_header_map_create() -> ffi::Arc<HeaderMap>;
    fn _pen_http_header_map_get(map: ffi::Arc<HeaderMap>, key: ffi::ByteString) -> ffi::ByteString;
    fn _pen_http_header_map_set(
        map: ffi::Arc<HeaderMap>,
        key: ffi::ByteString,
        value: ffi::ByteString,
    ) -> ffi::Arc<HeaderMap>;
}

impl HeaderMap {
    pub fn new() -> ffi::Arc<Self> {
        unsafe { _pen_http_header_map_create() }
    }

    pub fn get(this: &ffi::Arc<Self>, key: impl Into<ffi::ByteString>) -> ffi::ByteString {
        unsafe { _pen_http_header_map_get(this.clone(), key.into()) }
    }

    pub fn set(
        this: &ffi::Arc<Self>,
        key: impl Into<ffi::ByteString>,
        value: impl Into<ffi::ByteString>,
    ) -> ffi::Arc<Self> {
        unsafe { _pen_http_header_map_set(this.clone(), key.into(), value.into()) }
    }
}
