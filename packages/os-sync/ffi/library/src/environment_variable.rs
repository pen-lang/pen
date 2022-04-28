use crate::error::OsError;
use std::{ffi::OsString, str};

#[ffi::bindgen]
fn _pen_os_get_environment_variable(name: ffi::ByteString) -> Result<ffi::ByteString, OsError> {
    Ok(std::env::var(OsString::from(str::from_utf8(name.as_slice())?))?.into())
}
