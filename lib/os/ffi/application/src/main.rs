mod debug;
mod heap;
mod spawn;
mod unreachable;
mod utilities;

use std::process;
use tokio::runtime::Runtime;

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
    process::exit({
        // TODO When we remove this let statement, tasks are not awaited
        // on shutdown of the runtime somehow.
        let code: ffi::Number =
            Runtime::new()
                .unwrap()
                .block_on(ffi::async_closure(ffi::Arc::new(ffi::Closure::new(
                    _pen_os_main as *const u8,
                    (),
                ))));

        f64::from(code) as i32
    });
}
