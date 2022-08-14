mod concurrency;
mod debug;
mod heap;
mod unreachable;
mod utilities;

use std::time::Duration;
use tokio::time::sleep;

ffi::import!(_pen_main, async fn() -> ffi::None);

#[tokio::main]
async fn main() {
    _pen_main().await;

    // HACK Wait for all I/O buffers to be flushed (hopefully.)
    sleep(Duration::from_millis(50)).await;
}
