use crate::error::OsError;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn read(mut reader: impl AsyncReadExt + Unpin) -> Result<ffi::ByteString, OsError> {
    let mut buffer = vec![];

    reader.read_to_end(&mut buffer).await?;

    Ok(buffer.into())
}

pub async fn write(
    mut writer: impl AsyncWriteExt + Unpin,
    bytes: ffi::ByteString,
) -> Result<ffi::Number, OsError> {
    Ok(ffi::Number::new(
        writer.write(bytes.as_slice()).await? as f64,
    ))
}
