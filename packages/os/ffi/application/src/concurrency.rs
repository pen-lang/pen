use std::future::Future;
use tokio::{spawn, task::yield_now};

#[ffi::bindgen]
fn _pen_spawn(closure: ffi::Arc<ffi::Closure>) -> ffi::Arc<ffi::Closure> {
    ffi::future::to_closure(spawn_and_unwrap(ffi::future::from_closure(closure)))
}

async fn spawn_and_unwrap(future: impl Future<Output = ffi::Any> + Send + 'static) -> ffi::Any {
    spawn(future).await.unwrap()
}

#[ffi::bindgen]
async fn _pen_yield() {
    yield_now().await;
}
