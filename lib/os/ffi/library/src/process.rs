use std::{process::exit, time::Duration};
use tokio::time::sleep;

#[no_mangle]
extern "C" fn _pen_os_exit(
    stack: &mut ffi::cps::AsyncStack<ffi::Number>,
    _: ffi::cps::ContinuationFunction<ffi::None>,
    code: ffi::Number,
) -> ffi::cps::Result {
    // HACK Wait for all I/O buffers to be flushed (hopefully.)
    sleep(Duration::from_millis(50)).await;

    // Resolve a main function immediately with an exit code.
    exit(code)
}
