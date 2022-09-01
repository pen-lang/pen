use super::utilities;
use std::error::Error;
use tokio::io::{stderr, stdin, stdout, AsyncWriteExt};

#[ffi::bindgen]
async fn _pen_os_read_stdin() -> Result<ffi::ByteString, Box<dyn Error>> {
    utilities::read(&mut stdin()).await
}

#[ffi::bindgen]
async fn _pen_os_read_limit_stdin(limit: ffi::Number) -> Result<ffi::ByteString, Box<dyn Error>> {
    utilities::read_limit(&mut stdin(), f64::from(limit) as usize).await
}

#[ffi::bindgen]
async fn _pen_os_write_stdout(bytes: ffi::ByteString) -> Result<ffi::Number, Box<dyn Error>> {
    let count = utilities::write(&mut stdout(), bytes).await?;

    stdout().flush().await?;

    Ok(count)
}

#[ffi::bindgen]
async fn _pen_os_write_stderr(bytes: ffi::ByteString) -> Result<ffi::Number, Box<dyn Error>> {
    let count = utilities::write(&mut stderr(), bytes).await?;

    stderr().flush().await?;

    Ok(count)
}
