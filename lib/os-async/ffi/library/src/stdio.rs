use super::utilities;
use crate::{export_fn, result::FfiResult};
use std::{future::Future, pin::Pin, task::Poll};
use tokio::io::{stderr, stdin, stdout};

export_fn! {
    async fn _pen_os_read_stdin() -> ffi::Arc<FfiResult<ffi::ByteString>> {
        ffi::Arc::new(utilities::read(stdin()).await.into())
    }
}

#[no_mangle]
extern "C" fn _pen_os_read_limit_stdin(
    _limit: ffi::Number,
) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    todo!()
}

export_fn! {
    async fn _pen_os_write_stdout(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(utilities::write(stdout(), bytes).await.into())
    }
}

export_fn! {
    async fn _pen_os_write_stderr(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
        ffi::Arc::new(utilities::write(stderr(), bytes).await.into())
    }
}
