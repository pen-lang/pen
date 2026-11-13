#[unsafe(no_mangle)]
pub extern "C" fn _pen_unreachable() {
    unreachable!("PEN_TEST_UNREACHABLE_ERROR")
}
