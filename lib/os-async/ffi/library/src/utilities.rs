use crate::error::OsError;
use tokio::io::AsyncReadExt;

pub async fn read(reader: &mut (impl AsyncReadExt + Unpin)) -> Result<ffi::ByteString, OsError> {
    let mut buffer = vec![];

    reader.read_to_end(&mut buffer).await?;

    Ok(buffer.into())
}

/* pub fn write(writer: &mut impl Write + AsyncWrite, bytes: ffi::ByteString) -> Result<ffi::Number, OsError> {
    Ok(ffi::Number::new(
        writer.write(bytes.as_slice()).await? as f64,
    ))
} */
