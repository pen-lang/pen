#![feature(future_poll_fn)]

mod debug;
mod heap;
mod unreachable;
mod utilities;

use std::{
    future::poll_fn,
    task::{Context, Poll},
};

const INITIAL_STACK_CAPACITY: usize = 256;

#[cfg(not(test))]
#[link(name = "main")]
extern "C" {
    fn _pen_os_main(
        stack: &mut ffi::cps::AsyncStack,
        continuation: extern "C" fn(&mut ffi::cps::AsyncStack, f64) -> ffi::cps::Result,
    ) -> ffi::cps::Result;
}

#[cfg(test)]
unsafe extern "C" fn _pen_os_main(
    _: &mut ffi::cps::AsyncStack,
    _: extern "C" fn(&mut ffi::cps::AsyncStack, f64) -> ffi::cps::Result,
) -> ffi::cps::Result {
    ffi::cps::Result::new()
}

#[tokio::main]
async fn main() {
    let future = ffi::cps::Future::new(_pen_os_main, exit);
    let stack = ffi::cps::AsyncStack::new(INITIAL_STACK_CAPACITY);

    poll_fn(|context| {
        unsafe { _pen_os_main(&mut stack, exit) };

        unreachable!()
    })
    .await;

    unreachable!()
}

extern "C" fn exit(_: &mut ffi::cps::AsyncStack, code: f64) -> ffi::cps::Result {
    std::process::exit(code as i32)
}
