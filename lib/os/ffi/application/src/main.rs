mod debug;
mod heap;
mod spawn;
mod unreachable;
mod utilities;

type ContinuationFunction = ffi::cps::ContinuationFunction<ffi::None, ffi::None>;

#[cfg(not(test))]
#[link(name = "main")]
extern "C" {
    fn _pen_main(
        stack: &mut ffi::cps::AsyncStack<ffi::None>,
        continuation: ContinuationFunction,
    ) -> ffi::cps::Result;
}

#[cfg(test)]
extern "C" fn _pen_main(
    _: &mut ffi::cps::AsyncStack<ffi::None>,
    _: ContinuationFunction,
) -> ffi::cps::Result {
    ffi::cps::Result::new()
}

#[tokio::main]
async fn main() {
    let _: ffi::None = ffi::future::from_function(_pen_main).await;
}
