use crate::utilities;
use core::ops::DerefMut;
use std::sync::LazyLock;
use std::error::Error;
use tokio::{
    io::{stderr, stdin, stdout, AsyncWriteExt, Stderr, Stdin, Stdout},
    sync::Mutex,
};

// Use single buffers for standard I/O.
static STDIN: LazyLock<Mutex<Stdin>> = LazyLock::new(|| Mutex::new(stdin()));
static STDOUT: LazyLock<Mutex<Stdout>> = LazyLock::new(|| Mutex::new(stdout()));
static STDERR: LazyLock<Mutex<Stderr>> = LazyLock::new(|| Mutex::new(stderr()));

#[ffi::bindgen]
async fn _pen_os_read_stdin() -> Result<ffi::ByteString, Box<dyn Error>> {
    utilities::read(&mut STDIN.lock().await.deref_mut()).await
}

#[ffi::bindgen]
async fn _pen_os_read_limit_stdin(limit: ffi::Number) -> Result<ffi::ByteString, Box<dyn Error>> {
    utilities::read_limit(
        &mut STDIN.lock().await.deref_mut(),
        f64::from(limit) as usize,
    )
    .await
}

#[ffi::bindgen]
async fn _pen_os_write_stdout(bytes: ffi::ByteString) -> Result<ffi::Number, Box<dyn Error>> {
    let mut stdout = STDOUT.lock().await;
    let count = utilities::write(&mut *stdout, bytes).await?;

    stdout.flush().await?;

    Ok(count)
}

#[ffi::bindgen]
async fn _pen_os_write_stderr(bytes: ffi::ByteString) -> Result<ffi::Number, Box<dyn Error>> {
    let mut stderr = STDERR.lock().await;
    let count = utilities::write(&mut *stderr, bytes).await?;

    stderr.flush().await?;

    Ok(count)
}
