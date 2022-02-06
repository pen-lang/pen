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
    // TODO Remove these extra let expression and drop of runtime.
    // Without those codes, memory leak tests fail with an old version of a Rust
    // compiler.
    let runtime = Runtime::new().unwrap();
    let code: ffi::Number = runtime.block_on(async {
        let code = ffi::future::from_closure(ffi::Arc::new(ffi::Closure::new(
            _pen_os_main as *const u8,
            (),
        )))
        .await;

        stdout().flush().await.unwrap();
        stderr().flush().await.unwrap();

        code
    });

    drop(runtime);

    exit(f64::from(code) as i32);
}
