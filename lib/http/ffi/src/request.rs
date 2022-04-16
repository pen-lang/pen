#[ffi::any]
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    body: ffi::ByteString,
}

impl Request {
    pub fn new(body: impl Into<ffi::ByteString>) -> Self {
        Self { body: body.into() }
    }

    pub fn body(&self) -> ffi::ByteString {
        self.body.clone()
    }
}
