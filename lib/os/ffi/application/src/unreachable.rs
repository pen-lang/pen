#[no_mangle]
pub extern "C" fn _pen_unreachable() {
    unreachable!("PEN_OS_UNREACHABLE_ERROR")
}
