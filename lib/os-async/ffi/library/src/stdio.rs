use super::utilities;
use crate::result::FfiResult;
use std::future::Future;
use std::task::Poll;
use tokio::io::stdin;

// TODO Make those asynchronous.

#[no_mangle]
unsafe extern "C" fn _pen_os_read_stdin(
    stack: &mut ffi::cps::AsyncStack,
    continue_: ffi::cps::ContinuationFunction<ffi::Arc<FfiResult<ffi::ByteString>>>,
) -> ffi::cps::Result {
    let mut future = stack
        .restore()
        .unwrap_or_else(|| Box::pin(utilities::read(stdin())));

    match future.as_mut().poll(stack.context().unwrap()) {
        Poll::Ready(value) => continue_(stack, ffi::Arc::new(value.into())),
        Poll::Pending => {
            stack.suspend(_pen_os_read_stdin, continue_, future);
            ffi::cps::Result::new()
        }
    }
}

/* #[no_mangle]
extern "C" fn _pen_os_write_stdout(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(utilities::write(&mut stdout(), bytes).into())
}

#[no_mangle]
extern "C" fn _pen_os_write_stderr(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(utilities::write(&mut stderr(), bytes).into())
} */
