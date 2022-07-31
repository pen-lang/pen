ffi::import!(_pen_http_header_map_create, fn() -> HeaderMap);
ffi::import!(
    _pen_http_header_map_get,
    fn(map: HeaderMap, key: ffi::ByteString) -> ffi::ByteString
);
ffi::import!(
    _pen_http_header_map_set,
    fn(map: HeaderMap, key: ffi::ByteString, value: ffi::ByteString) -> HeaderMap
);
ffi::import!(_pen_http_header_map_keys, fn(map: HeaderMap) -> ffi::List);

#[repr(C)]
#[derive(Clone)]
pub struct HeaderMap(ffi::Arc<ffi::Any>);

impl HeaderMap {
    pub fn new() -> Self {
        unsafe { _pen_http_header_map_create() }
    }

    pub fn get(self, key: impl Into<ffi::ByteString>) -> ffi::ByteString {
        unsafe { _pen_http_header_map_get(self, key.into()) }
    }

    pub fn set(self, key: impl Into<ffi::ByteString>, value: impl Into<ffi::ByteString>) -> Self {
        unsafe { _pen_http_header_map_set(self, key.into(), value.into()) }
    }

    pub fn keys(self) -> ffi::List {
        unsafe { _pen_http_header_map_keys(self) }
    }
}
