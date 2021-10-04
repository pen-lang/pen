#[derive(Debug)]
#[repr(C)]
pub struct TestResult {
    is_error: ffi::Boolean,
    message: ffi::ByteString,
}

impl TestResult {
    pub fn is_error(&self) -> bool {
        usize::from(self.is_error) != 0
    }

    pub fn message(&self) -> String {
        String::from_utf8_lossy(self.message.as_slice()).into()
    }

    pub fn into_result(&self) -> Result<(), String> {
        if !self.is_error() {
            Ok(())
        } else {
            Err(self.message().into())
        }
    }
}
