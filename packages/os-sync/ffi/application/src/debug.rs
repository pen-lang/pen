use std::str;

#[no_mangle]
extern "C" fn _pen_debug(message: ffi::ByteString) -> ffi::None {
    eprintln!("{}", str::from_utf8(message.as_slice()).unwrap());

    ffi::None::new()
}
