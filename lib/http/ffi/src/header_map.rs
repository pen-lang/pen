#[ffi::any]
#[repr(C)]
#[derive(Clone)]
pub struct HeaderMap {
    headers: ffi::Any,
}

#[repr(C)]
struct FirstRest {
    ok: ffi::Boolean,
    first: ffi::Any,
    rest: ffi::Arc<ffi::List>,
}

extern "C" {
    fn _pen_http_header_map_create() -> ffi::Arc<HeaderMap>;
    fn _pen_http_header_map_get(map: ffi::Arc<HeaderMap>, key: ffi::ByteString) -> ffi::ByteString;
    fn _pen_http_header_map_set(
        map: ffi::Arc<HeaderMap>,
        key: ffi::ByteString,
        value: ffi::ByteString,
    ) -> ffi::Arc<HeaderMap>;
    fn _pen_http_header_map_keys(map: ffi::Arc<HeaderMap>) -> ffi::Arc<ffi::List>;

    fn _pen_http_first_rest(xs: ffi::Arc<ffi::List>) -> ffi::Arc<FirstRest>;
    fn _pen_http_to_string(xs: ffi::BoxAny) -> ffi::ByteString;
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

    pub fn iterate(
        this: &ffi::Arc<Self>,
        mut callback: impl FnMut(ffi::ByteString, ffi::ByteString),
    ) {
        let mut list = unsafe { _pen_http_header_map_keys(this.clone()) };

        loop {
            let first_rest = unsafe { _pen_http_first_rest(list.clone()) };

            if !bool::from(first_rest.ok) {
                break;
            }

            let key = unsafe { _pen_http_to_string(first_rest.first.clone().into()) };
            callback(key.clone(), Self::get(this, key));

            list = first_rest.rest.clone();
        }
    }
}
