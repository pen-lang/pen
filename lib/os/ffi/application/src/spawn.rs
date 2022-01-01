use std::{future::Future, pin::Pin, ptr, task::Poll};
use tokio::{spawn, task::JoinHandle};

type SpawnFuture = Pin<Box<JoinHandle<ffi::Any>>>;
type Storage = Option<ffi::Any>;
type Stack<'a> = ffi::cps::AsyncStack<'a, Storage>;
type ContinuationFunction = ffi::cps::ContinuationFunction<ffi::Any, Storage>;

extern "C" fn poll(
    stack: &mut Stack,
    continue_: ContinuationFunction,
    mut future: SpawnFuture,
) -> ffi::cps::Result {
    match future.as_mut().poll(stack.context().unwrap()) {
        Poll::Ready(value) => unsafe { continue_(stack, value.unwrap()) },
        Poll::Pending => {
            stack.suspend(resume, continue_, future);
            ffi::cps::Result::new()
        }
    }
}

extern "C" fn resume(stack: &mut Stack, continue_: ContinuationFunction) -> ffi::cps::Result {
    let future = stack.restore::<SpawnFuture>().unwrap();

    poll(stack, continue_, future)
}

extern "C" fn get_spawn_result(
    stack: &mut Stack,
    continue_: ContinuationFunction,
    future: *mut SpawnFuture,
) -> ffi::cps::Result {
    poll(stack, continue_, unsafe { ptr::read(future) })
}

#[no_mangle]
extern "C" fn _pen_spawn(closure: ffi::Arc<ffi::Closure>) -> ffi::Arc<ffi::Closure<SpawnFuture>> {
    ffi::Closure::new(
        get_spawn_result as *const u8,
        Box::pin(spawn(ffi::async_closure(closure))),
    )
    .into()
}
