#[ffi::any]
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    method: ffi::ByteString,
    uri: ffi::ByteString,
    body: ffi::ByteString,
}

impl Request {
    pub fn new(
        method: impl Into<ffi::ByteString>,
        uri: impl Into<ffi::ByteString>,
        body: impl Into<ffi::ByteString>,
    ) -> Self {
        Self {
            method: method.into(),
            uri: uri.into(),
            body: body.into(),
        }
    }

    pub fn method(&self) -> ffi::ByteString {
        self.method.clone()
    }

    pub fn uri(&self) -> ffi::ByteString {
        self.uri.clone()
    }

    pub fn body(&self) -> ffi::ByteString {
        self.body.clone()
    }
}
