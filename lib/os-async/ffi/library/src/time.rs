use crate::export_fn;
use std::time::Duration;
use tokio::time::sleep;

#[no_mangle]
extern "C" fn _pen_os_get_time() -> ffi::Number {
    todo!()
}

export_fn! {
    async fn _pen_os_sleep(milliseconds: ffi::Number) -> ffi::None {
        sleep(Duration::from_millis(f64::from(milliseconds) as u64)).await;
        ffi::None::new()
    }
}
