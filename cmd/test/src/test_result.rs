#[derive(Debug)]
#[repr(C)]
pub struct TestResult {
    is_error: ffi::Boolean,
    message: ffi::ByteString,
}

impl TestResult {
    #[allow(dead_code)]
    pub fn is_error(&self) -> bool {
        self.is_error.into()
    }

    #[allow(dead_code)]
    pub fn message(&self) -> String {
        String::from_utf8_lossy(self.message.as_slice()).into()
    }

    #[allow(dead_code)]
    pub fn to_result(&self) -> Result<(), String> {
        if !self.is_error() {
            Ok(())
        } else {
            Err(self.message())
        }
    }
}
