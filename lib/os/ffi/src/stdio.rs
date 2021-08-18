use super::utilities;
use crate::result::FfiResult;
use std::io::{stderr, stdin, stdout};

#[no_mangle]
extern "C" fn _pen_os_read_stdin() -> ffi::Arc<FfiResult<ffi::ByteString>> {
    ffi::Arc::new(utilities::read(&mut stdin()).into())
}

#[no_mangle]
extern "C" fn _pen_os_write_stdout(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(utilities::write(&mut stdout(), bytes).into())
}

#[no_mangle]
extern "C" fn _pen_os_write_stderr(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(utilities::write(&mut stderr(), bytes).into())
}
