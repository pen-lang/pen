use super::utilities;
use crate::error::OsError;
use std::io::{stderr, stdin, stdout};

#[ffi::bindgen]
fn _pen_os_read_stdin() -> Result<ffi::ByteString, OsError> {
    utilities::read(&mut stdin())
}

#[ffi::bindgen]
fn _pen_os_read_limit_stdin(limit: ffi::Number) -> Result<ffi::ByteString, OsError> {
    utilities::read_limit(&mut stdin(), f64::from(limit) as usize)
}

#[ffi::bindgen]
fn _pen_os_write_stdout(bytes: ffi::ByteString) -> Result<ffi::Number, OsError> {
    utilities::write(&mut stdout(), bytes)
}

#[ffi::bindgen]
fn _pen_os_write_stderr(bytes: ffi::ByteString) -> Result<ffi::Number, OsError> {
    utilities::write(&mut stderr(), bytes)
}
