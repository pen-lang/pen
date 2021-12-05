use crate::{error::OsError, result::FfiResult};
use std::{ffi::OsString, str};

#[ffi::bindgen]
fn _pen_os_get_environment_variable(name: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    ffi::Arc::new(get_environment_variable(name).into())
}

fn get_environment_variable(name: ffi::ByteString) -> Result<ffi::ByteString, OsError> {
    Ok(std::env::var(OsString::from(str::from_utf8(name.as_slice())?))?.into())
}
