use crate::header_map::HeaderMap;

ffi::import!(
    _pen_http_response_to_any,
    fn(response: Response) -> ffi::Any
);

#[repr(C)]
pub struct Response(ffi::Arc<ResponseInner>);

#[repr(C)]
struct ResponseInner {
    status: ffi::Number,
    headers: HeaderMap,
    body: ffi::ByteString,
}

impl Response {
    pub fn new(
        status: impl Into<ffi::Number>,
        headers: HeaderMap,
        body: impl Into<ffi::ByteString>,
    ) -> Self {
        Self(
            ResponseInner {
                status: status.into(),
                headers,
                body: body.into(),
            }
            .into(),
        )
    }

    pub fn status(&self) -> ffi::Number {
        self.0.status
    }

    pub fn headers(&self) -> HeaderMap {
        self.0.headers.clone()
    }

    pub fn body(&self) -> ffi::ByteString {
        self.0.body.clone()
    }
}

impl Into<ffi::Any> for Response {
    fn into(self) -> ffi::Any {
        unsafe { _pen_http_response_to_any(self) }
    }
}
