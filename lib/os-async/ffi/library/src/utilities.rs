use crate::error::OsError;
use std::io::{Read, Write};

// TODO Make those asynchronous.

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
