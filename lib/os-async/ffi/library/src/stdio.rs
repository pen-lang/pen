use super::utilities;
use crate::result::FfiResult;
use std::{future::Future, task::Poll};
use tokio::io::stdin;

// TODO Make those asynchronous.

#[no_mangle]
extern "C" fn _pen_read_stdin(
    stack: &mut ffi::cps::AsyncStack,
    continuation: extern "C" fn(
        &mut ffi::cps::AsyncStack,
        ffi::Arc<FfiResult<ffi::ByteString>>,
    ) -> ffi::cps::Result,
) -> ffi::cps::Result {
    let mut stdin = stdin();
    let mut future = if let Some(future) = stack.resume() {
        future
    } else {
        Box::pin(utilities::read(&mut stdin))
    };

    let value = match future.as_mut().poll(stack.context()) {
        Poll::Ready(value) => value,
        Poll::Pending => {
            stack.suspend(future);

            return ffi::cps::Result::new();
        }
    };

    continuation(stack, ffi::Arc::new(value.into()))
}

/* #[no_mangle]
extern "C" fn _pen_os_write_stdout(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(utilities::write(&mut stdout(), bytes).into())
}

#[no_mangle]
extern "C" fn _pen_os_write_stderr(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(utilities::write(&mut stderr(), bytes).into())
} */
