use std::{process::exit, time::Duration};
use tokio::time::sleep;

#[ffi::bindgen]
async fn _pen_os_exit(code: ffi::Number) -> ffi::None {
    // HACK Wait for all I/O buffers to be flushed (hopefully.)
    sleep(Duration::from_millis(50)).await;

    // Resolve a main function immediately with an exit code.
    exit(f64::from(code) as i32)
}
