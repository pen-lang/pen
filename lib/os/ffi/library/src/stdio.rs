use super::utilities;
use crate::result::FfiResult;
use core::ops::DerefMut;
use once_cell::sync::Lazy;
use tokio::{
    io::{stderr, stdin, stdout, Stderr, Stdin, Stdout},
    sync::Mutex,
};

static STDIN: Lazy<Mutex<Stdin>> = Lazy::new(|| Mutex::new(stdin()));
static STDOUT: Lazy<Mutex<Stdout>> = Lazy::new(|| Mutex::new(stdout()));
static STDERR: Lazy<Mutex<Stderr>> = Lazy::new(|| Mutex::new(stderr()));

#[ffi::bindgen]
async fn _pen_os_read_stdin() -> ffi::Arc<FfiResult<ffi::ByteString>> {
    ffi::Arc::new(utilities::read(&mut stdin()).await.into())
}

#[ffi::bindgen]
async fn _pen_os_read_limit_stdin(limit: ffi::Number) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    ffi::Arc::new(
        utilities::read_limit(STDIN.lock().await.deref_mut(), f64::from(limit) as usize)
            .await
            .into(),
    )
}

#[ffi::bindgen]
async fn _pen_os_write_stdout(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(
        utilities::write(STDOUT.lock().await.deref_mut(), bytes)
            .await
            .into(),
    )
}

#[ffi::bindgen]
async fn _pen_os_write_stderr(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(
        utilities::write(STDERR.lock().await.deref_mut(), bytes)
            .await
            .into(),
    )
}
