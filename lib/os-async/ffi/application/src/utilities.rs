use std::env;

const DEBUG_ENVIRONMENT_VARIABLE: &str = "PEN_DEBUG";
const OS_DEBUG_ENVIRONMENT_VARIABLE: &str = "PEN_OS_DEBUG";

pub fn is_debug() -> bool {
    env::var(DEBUG_ENVIRONMENT_VARIABLE).is_ok()
}

pub fn is_os_debug() -> bool {
    env::var(OS_DEBUG_ENVIRONMENT_VARIABLE).is_ok()
}
