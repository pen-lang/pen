use futures::future::poll_fn;
use std::future::Future;
use std::intrinsics::transmute;
use std::ptr;
use std::{pin::Pin, task::Poll};
use tokio::{spawn, task::JoinHandle};

type SpawnFuture = Pin<Box<JoinHandle<ffi::Any>>>;
type Storage = Option<ffi::Any>;
type Stack = ffi::cps::AsyncStack<Storage>;
type InitialStepFunction = unsafe extern "C" fn(
    stack: &mut Stack,
    continuation: ContinuationFunction,
    environment: &mut u8,
) -> ffi::cps::Result;
type StepFunction = ffi::cps::StepFunction<ffi::Any, Storage>;
type ContinuationFunction = ffi::cps::ContinuationFunction<ffi::Any, Storage>;

const INITIAL_STACK_SIZE: usize = 64;

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

extern "C" fn store_result(stack: &mut Stack, value: ffi::Any) -> ffi::cps::Result {
    *stack.storage_mut() = Some(value);

    ffi::cps::Result::new()
}

#[no_mangle]
pub extern "C" fn _pen_spawn(
    closure: ffi::Arc<ffi::Closure>,
) -> ffi::Arc<ffi::Closure<SpawnFuture>> {
    let handle = spawn(async move {
        let mut trampoline: Option<(StepFunction, ContinuationFunction)> = None;
        let mut stack = Stack::new(INITIAL_STACK_SIZE, None);

        poll_fn(move |context| {
            stack.set_context(context);

            if let Some((step, continue_)) = trampoline {
                unsafe { step(&mut stack, continue_) };
            } else {
                unsafe {
                    let entry_function =
                        transmute::<_, InitialStepFunction>(closure.entry_function());
                    entry_function(
                        &mut stack,
                        store_result,
                        &mut *(closure.payload() as *mut () as *mut u8),
                    )
                };
            }

            if let Some(value) = stack.storage() {
                value.clone().into()
            } else {
                trampoline = Some(stack.resume());
                Poll::Pending
            }
        })
        .await
    });

    ffi::Closure::new(get_spawn_result as *const u8, Box::pin(handle)).into()
}
