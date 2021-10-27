use crate::utilities::DEBUG_ENVIRONMENT_VARIABLE;
use std::str;

#[no_mangle]
pub extern "C" fn _pen_debug(message: ffi::ByteString) -> ffi::None {
    if std::env::var(DEBUG_ENVIRONMENT_VARIABLE).is_ok() {
        eprintln!("{}", str::from_utf8(message.as_slice()).unwrap());
    }

    ffi::None::new()
}
