#![feature(future_poll_fn)]

mod debug;
mod heap;
mod unreachable;
mod utilities;

use once_cell::sync::Lazy;
use std::{future::poll_fn, process, sync::Mutex, task::Poll};
use tokio::runtime::Runtime;

const INITIAL_STACK_CAPACITY: usize = 256;

#[cfg(not(test))]
#[link(name = "main")]
extern "C" {
    fn _pen_os_main(
        stack: &mut ffi::cps::AsyncStack,
        continuation: ffi::cps::ContinuationFunction<ffi::Number>,
    ) -> ffi::cps::Result;
}

#[cfg(test)]
unsafe extern "C" fn _pen_os_main(
    _: &mut ffi::cps::AsyncStack,
    _: ffi::cps::ContinuationFunction<ffi::Number>,
) -> ffi::cps::Result {
    ffi::cps::Result::new()
}

static EXIT_CODE: Lazy<Mutex<Option<i32>>> = Lazy::new(|| Mutex::new(None));

fn main() {
    process::exit({
        // TODO When we remove this let statement, tasks are not awaited
        // on shutdown of the runtime somehow.
        #[allow(clippy::let_and_return)]
        let code = Runtime::new().unwrap().block_on(async {
            let mut trampoline: (
                ffi::cps::StepFunction<ffi::Number>,
                ffi::cps::ContinuationFunction<ffi::Number>,
            ) = (_pen_os_main, exit);
            let mut stack = ffi::cps::AsyncStack::new(INITIAL_STACK_CAPACITY);

            poll_fn(move |context| {
                stack.set_context(context);

                let (step, continue_) = trampoline;
                unsafe { step(&mut stack, continue_) };

                if let Some(code) = *EXIT_CODE.lock().unwrap() {
                    code.into()
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

unsafe extern "C" fn exit(_: &mut ffi::cps::AsyncStack, code: ffi::Number) -> ffi::cps::Result {
    *EXIT_CODE.lock().unwrap() = (f64::from(code) as i32).into();

    ffi::cps::Result::new()
}
