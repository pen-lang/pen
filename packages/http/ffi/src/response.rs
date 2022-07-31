use crate::header_map::HeaderMap;

extern "C" {
    fn _pen_http_response_to_any(response: ffi::Arc<Response>) -> ffi::Any;
}

#[repr(C)]
pub struct Response {
    status: ffi::Number,
    headers: ffi::Arc<HeaderMap>,
    body: ffi::ByteString,
}

impl Response {
    pub fn new(
        status: impl Into<ffi::Number>,
        headers: ffi::Arc<HeaderMap>,
        body: impl Into<ffi::ByteString>,
    ) -> Self {
        Self {
            status: status.into(),
            headers,
            body: body.into(),
        }
    }

    pub fn status(&self) -> ffi::Number {
        self.status
    }

    pub fn headers(&self) -> ffi::Arc<HeaderMap> {
        self.headers.clone()
    }

    pub fn body(&self) -> ffi::ByteString {
        self.body.clone()
    }
}

impl Default for Response {
    fn default() -> Self {
        Self {
            status: Default::default(),
            headers: HeaderMap::new(),
            body: Default::default(),
        }
    }
}

impl Into<ffi::Any> for Response {
    fn into(self) -> ffi::Any {
        unsafe { _pen_http_response_to_any(self.into()) }
    }
}
