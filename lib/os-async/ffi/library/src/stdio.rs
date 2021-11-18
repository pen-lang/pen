use super::utilities;
use crate::result::FfiResult;
use std::io::{stderr, stdin, stdout};

// TODO Make those asynchronous.

#[no_mangle]
extern "C" fn _pen_os_read_stdin() -> ffi::Arc<FfiResult<ffi::ByteString>> {
    ffi::Arc::new(utilities::read(&mut stdin()).into())
}

#[no_mangle]
extern "C" fn _pen_os_read_limit_stdin(limit: ffi::Number) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    ffi::Arc::new(utilities::read_limit(&mut stdin(), f64::from(limit) as usize).into())
}

#[no_mangle]
extern "C" fn _pen_os_write_stdout(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(utilities::write(&mut stdout(), bytes).into())
}

#[no_mangle]
extern "C" fn _pen_os_write_stderr(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(utilities::write(&mut stderr(), bytes).into())
}
