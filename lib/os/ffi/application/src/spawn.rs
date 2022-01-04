use std::{future::Future, intrinsics::transmute};
use tokio::spawn;

#[no_mangle]
extern "C" fn _pen_spawn(closure: ffi::Arc<ffi::Closure>) -> ffi::Arc<ffi::Closure> {
    let closure = ffi::future::to_closure::<ffi::Any, _>(spawn_and_unwrap(
        ffi::future::from_closure(closure),
    ));

    unsafe { transmute(closure) }
}

async fn spawn_and_unwrap<F: Future<Output = ffi::Any> + Send + 'static>(future: F) -> ffi::Any {
    spawn(future).await.unwrap()
}
