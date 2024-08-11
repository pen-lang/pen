use std::sync::LazyLock;
use std::env;

static OS_DEBUG: Lazy<bool> = Lazy::new(|| env::var("PEN_OS_DEBUG").is_ok());

pub fn is_os_debug() -> bool {
    *OS_DEBUG
}
