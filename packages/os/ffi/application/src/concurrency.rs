use futures::FutureExt;
use once_cell::sync::Lazy;
use std::{
    future::Future,
    ops::Deref,
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use tokio::{
    select, spawn,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        Mutex,
    },
    task::yield_now,
    time::sleep,
};

type AnyFuture = Pin<Box<dyn Future<Output = ffi::Any> + Send>>;

const STOP_CHECK_INTERVAL: Duration = Duration::from_millis(50);

static SHOULD_STOP: AtomicBool = AtomicBool::new(false);

static FUTURE_CHANNEL: Lazy<(
    UnboundedSender<AnyFuture>,
    Mutex<UnboundedReceiver<AnyFuture>>,
)> = Lazy::new(|| {
    let (sender, receiver) = unbounded_channel();
    (sender, Mutex::new(receiver))
});

#[ffi::bindgen]
fn _pen_spawn(closure: ffi::Arc<ffi::Closure>) -> ffi::Arc<ffi::Closure> {
    let future = spawn_and_unwrap(ffi::future::from_closure(closure)).shared();

    let (sender, _) = FUTURE_CHANNEL.deref();
    // Ignore send errors due to channel close.
    sender.send(Box::pin(future.clone())).unwrap_or(());

    ffi::future::to_closure(future)
}

async fn spawn_and_unwrap(future: impl Future<Output = ffi::Any> + Send + 'static) -> ffi::Any {
    spawn(future).await.unwrap()
}

#[ffi::bindgen]
async fn _pen_yield() {
    yield_now().await;
}

pub async fn resolve_futures() {
    let (_, receiver) = FUTURE_CHANNEL.deref();
    let mut receiver = receiver.lock().await;

    loop {
        select! {
            future = receiver.recv() => {
                if let Some(future) = future {
                    future.await;
                } else {
                    break;
                }
            },
            _ = sleep(STOP_CHECK_INTERVAL) => {
                if SHOULD_STOP.load(Ordering::Relaxed) {
                    receiver.close();
                }
            }
        }
    }
}

pub fn stop_resolution() {
    SHOULD_STOP.store(true, Ordering::Relaxed);
}
