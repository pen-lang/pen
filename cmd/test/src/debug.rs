#[no_mangle]
pub extern "C" fn _pen_malloc(size: usize) -> ffi::None {
    if env::get("PEN_DEBUG").is_ok() {
        eprintln!("{}", str::from_utf8(message.as_slice()).unwrap());
    }

    ffi::None::new()
}
