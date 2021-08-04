use crate::error::OsError;
use std::io::{Read, Write};

pub fn read(reader: &mut impl Read) -> Result<ffi::ByteString, OsError> {
    let mut buffer = vec![];

    reader.read_to_end(&mut buffer)?;

    Ok(buffer.into())
}

pub fn write(writer: &mut impl Write, bytes: ffi::ByteString) -> Result<ffi::Number, OsError> {
    Ok(ffi::Number::new(writer.write(bytes.as_slice())? as f64))
}
