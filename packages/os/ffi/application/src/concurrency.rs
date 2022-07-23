use futures::future::FutureExt;
use tokio::{spawn, task::yield_now};

#[ffi::bindgen]
fn _pen_spawn(closure: ffi::Arc<ffi::Closure>) -> ffi::Arc<ffi::Closure> {
    ffi::future::to_closure(
        spawn(async move { ffi::future::from_closure::<_, ffi::Any>(closure).await })
            .map(Result::unwrap),
    )
}

#[ffi::bindgen]
async fn _pen_yield() {
    yield_now().await;
}
