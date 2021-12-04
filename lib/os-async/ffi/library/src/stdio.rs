use super::utilities;
use crate::result::FfiResult;
use tokio::io::{stderr, stdin, stdout};

#[ffi::bindgen]
async fn _pen_os_read_stdin() -> ffi::Arc<FfiResult<ffi::ByteString>> {
    ffi::Arc::new(utilities::read(stdin()).await.into())
}

#[no_mangle]
extern "C" fn _pen_os_read_limit_stdin(
    _limit: ffi::Number,
) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    todo!()
}

#[ffi::bindgen]
async fn _pen_os_write_stdout(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(utilities::write(stdout(), bytes).await.into())
}

#[ffi::bindgen]
async fn _pen_os_write_stderr(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(utilities::write(stderr(), bytes).await.into())
}
