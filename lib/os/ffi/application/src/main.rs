mod concurrency;
mod debug;
mod heap;
mod unreachable;
mod utilities;

use std::time::Duration;
use tokio::time::sleep;

type ContinuationFunction = ffi::cps::ContinuationFunction<ffi::None, ffi::None>;

#[cfg(not(test))]
#[link(name = "main")]
extern "C" {
    fn _pen_main(
        stack: &mut ffi::cps::AsyncStack<ffi::None>,
        continuation: ContinuationFunction,
    ) -> ffi::cps::Result;
}

#[cfg(test)]
extern "C" fn _pen_main(
    _: &mut ffi::cps::AsyncStack<ffi::None>,
    _: ContinuationFunction,
) -> ffi::cps::Result {
    ffi::cps::Result::new()
}

#[tokio::main]
async fn main() {
    ffi::future::from_function(_pen_main).await;

    // HACK Wait for all I/O buffers to be flushed (hopefully.)
    sleep(Duration::from_millis(50)).await;
}
