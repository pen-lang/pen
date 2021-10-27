use std::{env, str};

#[no_mangle]
pub extern "C" fn _pen_debug(message: ffi::ByteString) -> ffi::None {
    if env::var("PEN_DEBUG").is_ok() {
        eprintln!("{}", str::from_utf8(message.as_slice()).unwrap());
    }

    ffi::None::new()
}
