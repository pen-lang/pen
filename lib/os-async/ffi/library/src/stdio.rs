use super::utilities;
use crate::result::FfiResult;
use tokio::io::{stderr, stdin, stdout};

#[ffi::bindgen]
async fn _pen_os_read_stdin() -> ffi::Arc<FfiResult<ffi::ByteString>> {
    ffi::Arc::new(utilities::read(&mut stdin()).await.into())
}

#[ffi::bindgen]
async fn _pen_os_read_limit_stdin(limit: ffi::Number) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    ffi::Arc::new(
        utilities::read_limit(&mut stdin(), f64::from(limit) as usize)
            .await
            .into(),
    )
}

#[ffi::bindgen]
async fn _pen_os_write_stdout(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(utilities::write(&mut stdout(), bytes).await.into())
}

#[ffi::bindgen]
async fn _pen_os_write_stderr(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(utilities::write(&mut stderr(), bytes).await.into())
}
