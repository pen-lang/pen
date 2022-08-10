use std::{error::Error, ffi::OsString, str};

#[ffi::bindgen]
fn _pen_os_get_environment_variable(
    name: ffi::ByteString,
) -> Result<ffi::ByteString, Box<dyn Error>> {
    Ok(std::env::var(OsString::from(str::from_utf8(name.as_slice())?))?.into())
}
