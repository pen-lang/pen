#[ffi::bindgen]
fn _pen_unreachable() -> ffi::None {
    unreachable!("PEN_OS_UNREACHABLE_ERROR")
}

#[ffi::bindgen]
fn _pen_os_unreachable() {
    _pen_unreachable()
}
