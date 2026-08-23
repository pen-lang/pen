use std::sync::LazyLock;
use tokio::runtime::Runtime;

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

pub fn runtime() -> &'static Runtime {
    &RUNTIME
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    #[test]
    fn spawn_task_outside_runtime_context() {
        assert_eq!(block_on(runtime().spawn(async { 42 })).unwrap(), 42);
    }
}
