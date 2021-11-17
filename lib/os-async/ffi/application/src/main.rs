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
        stack: *mut ffi::cps::Stack,
        continuation: extern "C" fn(*mut ffi::cps::Stack, f64) -> ffi::cps::Result,
    ) -> ffi::cps::Result;
}

#[cfg(test)]
unsafe extern "C" fn _pen_os_main(
    _: *mut ffi::cps::Stack,
    _: extern "C" fn(*mut ffi::cps::Stack, f64) -> ffi::cps::Result,
) -> ffi::cps::Result {
    ffi::cps::Result::new()
}

#[tokio::main]
async fn main() {
    poll_fn(main_wrapper).await;

    unreachable!()
}

fn main_wrapper<'a, 'b>(_context: &'a mut Context<'b>) -> Poll<()> {
    let mut stack = ffi::cps::Stack::new(INITIAL_STACK_CAPACITY);

    unsafe { _pen_os_main(&mut stack, exit) };

    unreachable!()
}

extern "C" fn exit(_: *mut ffi::cps::Stack, code: f64) -> ffi::cps::Result {
    std::process::exit(code as i32)
}
