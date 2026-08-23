use core::{
    future::{poll_fn, Future},
    pin::pin,
};
use std::sync::LazyLock;
use tokio::runtime::{Builder, Runtime};

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap()
});

// Polls a future in the runtime context.
pub async fn run<T>(future: impl Future<Output = T>) -> T {
    let mut future = pin!(future);

    poll_fn(|context| {
        let _guard = RUNTIME.enter();

        future.as_mut().poll(context)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use tokio::runtime::Handle;

    #[test]
    fn enter_context_on_poll() {
        assert!(block_on(run(async { Handle::try_current().is_ok() })));
    }

    #[test]
    fn leave_context_after_poll() {
        block_on(run(async {}));

        assert!(Handle::try_current().is_err());
    }

    mod attribute {
        use super::*;

        #[pen_ffi_macro::runtime(crate = "crate")]
        async fn is_in_context() -> bool {
            Handle::try_current().is_ok()
        }

        #[pen_ffi_macro::runtime(crate = "crate")]
        async fn add(x: f64, y: f64) -> Result<f64, ()> {
            let x = Ok::<_, ()>(x)?;

            Ok(x + y)
        }

        #[test]
        fn enter_context() {
            assert!(block_on(is_in_context()));
        }

        #[test]
        fn leave_context() {
            block_on(is_in_context());

            assert!(Handle::try_current().is_err());
        }

        #[test]
        fn pass_arguments() {
            assert_eq!(block_on(add(40.0, 2.0)), Ok(42.0));
        }
    }
}
