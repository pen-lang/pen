use crate::{
    error::{READ_FILE_ERROR, WRITE_FILE_ERROR},
    result::FfiResult,
};
use std::io::{stderr, stdin, stdout, Read, Write};

#[no_mangle]
extern "C" fn _pen_os_read_stdin() -> ffi::Arc<FfiResult<ffi::ByteString>> {
    let mut buffer = vec![];

    match stdin().read_to_end(&mut buffer) {
        Ok(_) => FfiResult::ok(buffer.into()),
        Err(_) => FfiResult::error(READ_FILE_ERROR),
    }
    .into()
}

#[no_mangle]
extern "C" fn _pen_os_write_stdout(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    write(&mut stdout(), bytes)
}

#[no_mangle]
extern "C" fn _pen_os_write_stderr(bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    write(&mut stderr(), bytes)
}

fn write(writable: &mut impl Write, bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    match writable.write(bytes.as_slice()) {
        Ok(count) => FfiResult::ok(ffi::Number::new(count as f64)),
        Err(_) => FfiResult::error(WRITE_FILE_ERROR),
    }
    .into()
}
