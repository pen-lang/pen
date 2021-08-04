use crate::error::{READ_FILE_ERROR, WRITE_FILE_ERROR};
use std::io::{Read, Write};

pub fn read(reader: &mut impl Read) -> Result<ffi::ByteString, f64> {
    let mut buffer = vec![];

    reader
        .read_to_end(&mut buffer)
        .map_err(|_| READ_FILE_ERROR)?;

    Ok(buffer.into())
}

pub fn write(writer: &mut impl Write, bytes: ffi::ByteString) -> Result<ffi::Number, f64> {
    Ok(ffi::Number::new(
        writer
            .write(bytes.as_slice())
            .map_err(|_| WRITE_FILE_ERROR)? as f64,
    ))
}
