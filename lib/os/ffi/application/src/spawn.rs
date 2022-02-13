use std::future::Future;
use tokio::spawn;

#[no_mangle]
extern "C" fn _pen_spawn(closure: ffi::Arc<ffi::Closure>) -> ffi::Arc<ffi::Closure> {
    ffi::future::to_closure(spawn_and_unwrap(ffi::future::from_closure(closure)))
}

async fn spawn_and_unwrap(future: impl Future<Output = ffi::Any> + Send + 'static) -> ffi::Any {
    spawn(future).await.unwrap()
}

#[no_mangle]
extern "C" fn _pen_join(xs: ffi::Arc<ffi::List>) -> ffi::Arc<ffi::List> {
    unsafe { _pen_os_join(xs) }
}
