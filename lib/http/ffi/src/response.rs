#[ffi::any]
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Response {
    status: ffi::Number,
    body: ffi::ByteString,
}

impl Response {
    pub fn new(status: impl Into<ffi::Number>, body: impl Into<ffi::ByteString>) -> Self {
        Self {
            status: status.into(),
            body: body.into(),
        }
    }

    pub fn status(&self) -> ffi::Number {
        self.status
    }

    pub fn body(&self) -> ffi::ByteString {
        self.body.clone()
    }
}
