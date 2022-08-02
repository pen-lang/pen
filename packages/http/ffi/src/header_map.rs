#[repr(C)]
#[derive(Clone)]
pub struct HeaderMap(ffi::Arc<ffi::Any>);

impl HeaderMap {
    pub fn new() -> Self {
        ffi::import!(_pen_http_header_map_create, fn() -> HeaderMap);

        unsafe { _pen_http_header_map_create() }
    }

    pub fn get(&self, key: impl Into<ffi::ByteString>) -> ffi::ByteString {
        ffi::import!(
            _pen_http_header_map_get,
            fn(map: HeaderMap, key: ffi::ByteString) -> ffi::ByteString
        );

        unsafe { _pen_http_header_map_get(self.clone(), key.into()) }
    }

    pub fn set(&self, key: impl Into<ffi::ByteString>, value: impl Into<ffi::ByteString>) -> Self {
        ffi::import!(
            _pen_http_header_map_set,
            fn(map: HeaderMap, key: ffi::ByteString, value: ffi::ByteString) -> HeaderMap
        );

        unsafe { _pen_http_header_map_set(self.clone(), key.into(), value.into()) }
    }

    pub fn keys(&self) -> ffi::List {
        ffi::import!(_pen_http_header_map_keys, fn(map: HeaderMap) -> ffi::List);

        unsafe { _pen_http_header_map_keys(self.clone()) }
    }
}

impl Default for HeaderMap {
    fn default() -> Self {
        Self::new()
    }
}
