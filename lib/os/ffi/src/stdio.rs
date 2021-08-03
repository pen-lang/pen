use super::utilities;
use crate::result::FfiResult;
use std::io::{stderr, stdin, stdout};

#[no_mangle]
extern "C" fn _pen_os_read_stdin() -> ffi::Arc<FfiResult<ffi::ByteString>> {
    utilities::read(&mut stdin())
}

#[no_mangle]
extern "C" fn _pen_os_write_stdout(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    utilities::write(&mut stdout(), bytes)
}

#[no_mangle]
extern "C" fn _pen_os_write_stderr(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    utilities::write(&mut stderr(), bytes)
}
