mod debug;
mod heap;
mod spawn;
mod unreachable;
mod utilities;

type Stack = ffi::cps::AsyncStack<()>;
type ContinuationFunction = ffi::cps::ContinuationFunction<ffi::None, ()>;

#[cfg(not(test))]
#[link(name = "main")]
extern "C" {
    fn _pen_main(stack: &mut Stack, continuation: ContinuationFunction) -> ffi::cps::Result;
}

#[cfg(test)]
unsafe extern "C" fn _pen_main(_: &mut Stack, _: ContinuationFunction) -> ffi::cps::Result {
    ffi::cps::Result::new()
}

#[tokio::main]
async fn main() {
    let _: ffi::None =
        ffi::future::from_closure(ffi::Arc::new(ffi::Closure::new(_pen_main as *const u8, ())))
            .await;
}
