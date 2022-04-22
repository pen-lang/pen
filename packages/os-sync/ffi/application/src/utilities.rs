use once_cell::sync::Lazy;
use std::env;

static DEBUG: Lazy<bool> = Lazy::new(|| env::var("PEN_DEBUG").is_ok());
static OS_DEBUG: Lazy<bool> = Lazy::new(|| env::var("PEN_OS_DEBUG").is_ok());

pub fn is_debug() -> bool {
    *DEBUG
}

pub fn is_os_debug() -> bool {
    *OS_DEBUG
}
