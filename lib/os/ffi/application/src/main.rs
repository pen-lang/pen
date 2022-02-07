mod debug;
mod heap;
mod spawn;
mod unreachable;
mod utilities;

use std::process::exit;
use tokio::{
    io::{stderr, stdout, AsyncWriteExt},
    runtime::Runtime,
};

type ExitCode = ffi::Number;
type Stack = ffi::cps::AsyncStack<Option<ExitCode>>;
type ContinuationFunction = ffi::cps::ContinuationFunction<ExitCode, Option<ExitCode>>;

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
    // TODO Allocate a closure in an async block below instead.
    // Without this extra variable definition, memory leak tests fail somehow.
    let closure = ffi::Arc::new(ffi::Closure::new(_pen_os_main as *const u8, ()));
    let code: ffi::Number = Runtime::new().unwrap().block_on(async {
        let code = ffi::future::from_closure(closure).await;

        stdout().flush().await.unwrap();
        stderr().flush().await.unwrap();

        code
    });

    exit(f64::from(code) as i32);
}
