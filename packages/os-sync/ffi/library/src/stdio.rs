use super::utilities;
use std::{
    error::Error,
    io::{stderr, stdin, stdout},
};

#[ffi::bindgen]
fn _pen_os_read_stdin() -> Result<ffi::ByteString, Box<dyn Error>> {
    utilities::read(&mut stdin())
}

#[ffi::bindgen]
fn _pen_os_read_limit_stdin(limit: ffi::Number) -> Result<ffi::ByteString, Box<dyn Error>> {
    utilities::read_limit(&mut stdin(), f64::from(limit) as usize)
}

#[ffi::bindgen]
fn _pen_os_write_stdout(bytes: ffi::ByteString) -> Result<ffi::Number, Box<dyn Error>> {
    utilities::write(&mut stdout(), bytes)
}

#[ffi::bindgen]
fn _pen_os_write_stderr(bytes: ffi::ByteString) -> Result<ffi::Number, Box<dyn Error>> {
    utilities::write(&mut stderr(), bytes)
}
