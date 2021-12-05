use std::{
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[ffi::bindgen]
fn _pen_os_get_time() -> ffi::Number {
    (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|error| error.duration())
        .as_millis() as f64)
        .into()
}

#[ffi::bindgen]
fn _pen_os_sleep(milliseconds: ffi::Number) {
    sleep(Duration::from_millis(f64::from(milliseconds) as u64));
}
