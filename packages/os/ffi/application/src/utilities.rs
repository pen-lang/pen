use std::{env, sync::LazyLock};

static OS_DEBUG: LazyLock<bool> = LazyLock::new(|| env::var("PEN_OS_DEBUG").is_ok());

pub fn is_os_debug() -> bool {
    *OS_DEBUG
}
