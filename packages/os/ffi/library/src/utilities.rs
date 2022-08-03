use std::{error::Error, str};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn read(
    reader: &mut (impl AsyncReadExt + Unpin),
) -> Result<ffi::ByteString, Box<dyn Error>> {
    let mut buffer = vec![];

    reader.read_to_end(&mut buffer).await?;

    Ok(buffer.into())
}

pub async fn read_limit(
    reader: &mut (impl AsyncReadExt + Unpin),
    limit: usize,
) -> Result<ffi::ByteString, Box<dyn Error>> {
    let mut buffer = vec![0; limit];
    let size = reader.read(&mut buffer).await?;

    buffer.truncate(size);

    Ok(buffer.into())
}

pub async fn write(
    writer: &mut (impl AsyncWriteExt + Unpin),
    bytes: ffi::ByteString,
) -> Result<ffi::Number, Box<dyn Error>> {
    Ok(ffi::Number::new(
        writer.write(bytes.as_slice()).await? as f64,
    ))
}

pub fn decode_path(path: &ffi::ByteString) -> Result<&str, Box<dyn Error>> {
    Ok(str::from_utf8(path.as_slice())?)
}
