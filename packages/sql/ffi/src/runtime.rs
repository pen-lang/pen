use std::{
    future::Future,
    pin::Pin,
    sync::LazyLock,
    task::{Context, Poll},
};
use tokio::runtime::{Builder, Runtime};

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap()
});

// Polls and drops a future in the runtime context. sqlx spawns tasks even on
// drop of pool connections.
pub fn run<F: Future>(future: F) -> impl Future<Output = F::Output> {
    Run(Some(Box::pin(future)))
}

struct Run<F>(Option<Pin<Box<F>>>);

impl<F: Future> Future for Run<F> {
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
        let _guard = RUNTIME.enter();

        self.0.as_mut().unwrap().as_mut().poll(context)
    }
}

impl<F> Drop for Run<F> {
    fn drop(&mut self) {
        let _guard = RUNTIME.enter();

        self.0.take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use tokio::runtime::Handle;

    struct Probe(Arc<AtomicBool>);

    impl Drop for Probe {
        fn drop(&mut self) {
            self.0
                .store(Handle::try_current().is_ok(), Ordering::SeqCst);
        }
    }

    #[test]
    fn enter_context_on_poll() {
        assert!(block_on(run(async { Handle::try_current().is_ok() })));
    }

    #[test]
    fn leave_context_after_poll() {
        block_on(run(async {}));

        assert!(Handle::try_current().is_err());
    }

    #[test]
    fn enter_context_on_drop() {
        let dropped = Arc::new(AtomicBool::new(false));
        let probe = Probe(dropped.clone());

        drop(run(async move { drop(probe) }));

        assert!(dropped.load(Ordering::SeqCst));
    }
}
