use std::{
    error::Error,
    io::{Read, Write},
    str,
};

pub fn read(reader: &mut impl Read) -> Result<ffi::ByteString, Box<dyn Error>> {
    let mut buffer = vec![];

    reader.read_to_end(&mut buffer)?;

    Ok(buffer.into())
}

pub fn read_limit(reader: &mut impl Read, limit: usize) -> Result<ffi::ByteString, Box<dyn Error>> {
    let mut buffer = vec![0; limit];
    let size = reader.read(&mut buffer)?;

    buffer.truncate(size);

    Ok(buffer.into())
}

pub fn write(
    writer: &mut impl Write,
    bytes: ffi::ByteString,
) -> Result<ffi::Number, Box<dyn Error>> {
    Ok(ffi::Number::new(writer.write(bytes.as_slice())? as f64))
}

pub fn decode_path(path: &ffi::ByteString) -> Result<&str, Box<dyn Error>> {
    Ok(str::from_utf8(path.as_slice())?)
}
