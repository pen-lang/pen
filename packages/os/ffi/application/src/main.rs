mod concurrency;
mod debug;
mod heap;
mod unreachable;
mod utilities;

use concurrency::{resolve_futures, stop_resolution};
use std::time::Duration;
use tokio::{spawn, time::sleep};

#[cfg(not(test))]
#[link(name = "main")]
extern "C" {
    ffi::import!(_pen_main, fn() -> ffi::None);
}

#[cfg(test)]
extern "C" fn _pen_main(
    _: &mut ffi::cps::AsyncStack<ffi::None>,
    _: ffi::cps::ContinuationFunction<ffi::None, ffi::None>,
) {
}

#[tokio::main]
async fn main() {
    let children = spawn(resolve_futures());

    ffi::future::from_function(_pen_main).await;

    stop_resolution();
    children.await.unwrap();

    // HACK Wait for all I/O buffers to be flushed (hopefully.)
    sleep(Duration::from_millis(50)).await;
}
