use crate::utilities::is_debug;
use std::str;

#[no_mangle]
pub extern "C" fn _pen_debug(message: ffi::ByteString) -> ffi::None {
    if is_debug() {
        eprintln!("{}", str::from_utf8(message.as_slice()).unwrap());
    }

    ffi::None::new()
}
