mod debug;
mod heap;
mod spawn;
mod unreachable;
mod utilities;

use std::time::Duration;
use tokio::time::sleep;

type ExitCode = ffi::Number;
type Stack = ffi::cps::AsyncStack<Option<ExitCode>>;
type ContinuationFunction = ffi::cps::ContinuationFunction<ExitCode, Option<ExitCode>>;

#[cfg(not(test))]
#[link(name = "main")]
extern "C" {
    fn _pen_main(stack: &mut Stack, continuation: ContinuationFunction) -> ffi::cps::Result;
}

#[cfg(test)]
unsafe extern "C" fn _pen_main(_: &mut Stack, _: ContinuationFunction) -> ffi::cps::Result {
    ffi::cps::Result::new()
}

#[tokio::main]
async fn main() {
    let _: ffi::None =
        ffi::future::from_closure(ffi::Arc::new(ffi::Closure::new(_pen_main as *const u8, ())))
            .await;

    // HACK Wait for all I/O buffers to be flushed (hopefully.)
    sleep(Duration::from_millis(50)).await;
}
