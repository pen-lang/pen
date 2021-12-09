use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

#[ffi::bindgen]
fn _pen_os_get_time() -> ffi::Number {
    (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|error| error.duration())
        .as_millis() as f64)
        .into()
}

#[ffi::bindgen]
async fn _pen_os_sleep(milliseconds: ffi::Number) {
    sleep(Duration::from_millis(f64::from(milliseconds) as u64)).await;
}
