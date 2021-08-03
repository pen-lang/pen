use crate::{
    error::{READ_FILE_ERROR, WRITE_FILE_ERROR},
    result::FfiResult,
};
use std::io::{Read, Write};

pub fn read(reader: &mut impl Read) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    let mut buffer = vec![];

    match reader.read_to_end(&mut buffer) {
        Ok(_) => FfiResult::ok(buffer.into()),
        Err(_) => FfiResult::error(READ_FILE_ERROR),
    }
    .into()
}

pub fn write(writer: &mut impl Write, bytes: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Number>> {
    match writer.write(bytes.as_slice()) {
        Ok(count) => FfiResult::ok(ffi::Number::new(count as f64)),
        Err(_) => FfiResult::error(WRITE_FILE_ERROR),
    }
    .into()
}
