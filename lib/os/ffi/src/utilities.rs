use crate::error::OsError;
use std::{
    env,
    io::{Read, Write},
    str,
};

const DEBUG_ENVIRONMENT_VARIABLE: &str = "PEN_DEBUG";
const OS_DEBUG_ENVIRONMENT_VARIABLE: &str = "PEN_OS_DEBUG";

pub fn is_debug() -> bool {
    env::var(DEBUG_ENVIRONMENT_VARIABLE).is_ok()
}

pub fn is_os_debug() -> bool {
    env::var(OS_DEBUG_ENVIRONMENT_VARIABLE).is_ok()
}

pub fn read(reader: &mut impl Read) -> Result<ffi::ByteString, OsError> {
    let mut buffer = vec![];

    reader.read_to_end(&mut buffer)?;

    Ok(buffer.into())
}

pub fn read_limit(reader: &mut impl Read, limit: usize) -> Result<ffi::ByteString, OsError> {
    let mut buffer = vec![0; limit];
    let size = reader.read(&mut buffer)?;

    buffer.truncate(size);

    Ok(buffer.into())
}

pub fn write(writer: &mut impl Write, bytes: ffi::ByteString) -> Result<ffi::Number, OsError> {
    Ok(ffi::Number::new(writer.write(bytes.as_slice())? as f64))
}

pub fn decode_path(path: &ffi::ByteString) -> Result<&str, OsError> {
    Ok(str::from_utf8(path.as_slice())?)
}
