#[ffi::any]
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct Response {
    status: ffi::Number,
    body: ffi::ByteString,
}

impl Response {
    pub fn status(&self) -> ffi::Number {
        self.status
    }

    pub fn body(&self) -> ffi::ByteString {
        self.body.clone()
    }
}
