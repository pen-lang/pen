mod debug;
mod heap;
mod spawn;
mod unreachable;
mod utilities;

use futures::future::poll_fn;
use std::{process, task::Poll};
use tokio::runtime::Runtime;

type ExitCode = ffi::Number;
type Stack = ffi::cps::AsyncStack<Option<ExitCode>>;
type StepFunction = ffi::cps::StepFunction<ExitCode, Option<ExitCode>>;
type ContinuationFunction = ffi::cps::ContinuationFunction<ExitCode, Option<ExitCode>>;

const INITIAL_STACK_CAPACITY: usize = 256;

#[cfg(not(test))]
#[link(name = "main")]
extern "C" {
    fn _pen_os_main(stack: &mut Stack, continuation: ContinuationFunction) -> ffi::cps::Result;
}

#[cfg(test)]
unsafe extern "C" fn _pen_os_main(_: &mut Stack, _: ContinuationFunction) -> ffi::cps::Result {
    ffi::cps::Result::new()
}

fn main() {
    process::exit({
        // TODO When we remove this let statement, tasks are not awaited
        // on shutdown of the runtime somehow.
        #[allow(clippy::let_and_return)]
        let code = Runtime::new().unwrap().block_on(async {
            let mut trampoline: (StepFunction, ContinuationFunction) = (_pen_os_main, exit);
            let mut stack = Stack::new(INITIAL_STACK_CAPACITY, None);

            poll_fn(move |context| {
                stack.set_context(context);

                let (step, continue_) = trampoline;
                // TODO Pass a closure environment (or a null pointer) for the first call.
                unsafe { step(&mut stack, continue_) };

                if let Some(code) = *stack.storage() {
                    (f64::from(code) as i32).into()
                } else {
                    trampoline = stack.resume();
                    Poll::Pending
                }
            })
            .await
        });

        code
    });
}

unsafe extern "C" fn exit(stack: &mut Stack, code: ffi::Number) -> ffi::cps::Result {
    *stack.storage_mut() = Some(code);

    ffi::cps::Result::new()
}
