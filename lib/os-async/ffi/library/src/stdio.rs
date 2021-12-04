use super::utilities;
use crate::{error::OsError, result::FfiResult};
use std::{future::Future, pin::Pin, task::Poll};
use tokio::io::{stdin, stdout};

type PinnedFuture<T> = Pin<Box<dyn Future<Output = T>>>;

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

#[no_mangle]
extern "C" fn _pen_os_read_limit_stdin(
    _limit: ffi::Number,
) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    todo!()
}

type WriteStdoutFuture = PinnedFuture<Result<ffi::Number, OsError>>;

#[no_mangle]
unsafe extern "C" fn _pen_os_write_stdout(
    stack: &mut ffi::cps::AsyncStack,
    continue_: ffi::cps::ContinuationFunction<ffi::Arc<FfiResult<ffi::Number>>>,
    bytes: ffi::ByteString,
) -> ffi::cps::Result {
    let mut future: WriteStdoutFuture = Box::pin(utilities::write(stdout(), bytes));

    match future.as_mut().poll(stack.context().unwrap()) {
        Poll::Ready(value) => continue_(stack, ffi::Arc::new(value.into())),
        Poll::Pending => {
            stack.suspend(_pen_os_write_stdout_poll, continue_, future);
            ffi::cps::Result::new()
        }
    }
}

unsafe extern "C" fn _pen_os_write_stdout_poll(
    stack: &mut ffi::cps::AsyncStack,
    continue_: ffi::cps::ContinuationFunction<ffi::Arc<FfiResult<ffi::Number>>>,
) -> ffi::cps::Result {
    let mut future: WriteStdoutFuture = stack.restore().unwrap();

    match future.as_mut().poll(stack.context().unwrap()) {
        Poll::Ready(value) => continue_(stack, ffi::Arc::new(value.into())),
        Poll::Pending => {
            stack.suspend(_pen_os_write_stdout_poll, continue_, future);
            ffi::cps::Result::new()
        }
    }
}

#[no_mangle]
extern "C" fn _pen_os_write_stderr(_bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    todo!()
}
