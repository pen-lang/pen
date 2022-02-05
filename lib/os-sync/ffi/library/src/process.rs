use std::process::exit;

#[ffi::bindgen]
fn _pen_os_exit(code: ffi::Number) -> ffi::None {
    exit(f64::from(code) as i32)
}
