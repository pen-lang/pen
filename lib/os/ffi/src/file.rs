use super::open_file_options::OpenFileOptions;
use crate::result::FfiResult;
use std::{
    fs::{File, OpenOptions},
    io::Read,
    io::Write,
    ops::Deref,
    path::Path,
    str,
    sync::{Arc, LockResult, RwLock, RwLockWriteGuard},
};

const UTF8_DECODE_ERROR: f64 = 0.0;
const OPEN_FILE_ERROR: f64 = 1.0;
const LOCK_FILE_ERROR: f64 = 2.0;
const READ_FILE_ERROR: f64 = 3.0;
const WRITE_FILE_ERROR: f64 = 4.0;

#[derive(Clone, Debug)]
pub struct FfiFile {
    file: Arc<RwLock<File>>,
}

ffi::type_information!(ffi_file, crate::file::FfiFile);

impl FfiFile {
    pub fn new(file: File) -> Self {
        Self {
            file: Arc::new(RwLock::new(file)),
        }
    }

    pub fn get_mut(&self) -> LockResult<RwLockWriteGuard<'_, File>> {
        self.file.write()
    }
}

impl From<File> for FfiFile {
    fn from(file: File) -> Self {
        Self::new(file)
    }
}

#[no_mangle]
extern "C" fn _pen_os_open_file(
    path: ffi::ByteString,
    options: ffi::Arc<OpenFileOptions>,
) -> ffi::Arc<FfiResult<ffi::Any>> {
    FfiResult::ok(unsafe {
        FfiFile::new(
            match OpenOptions::from(options.deref()).open(&Path::new(&match str::from_utf8(
                path.as_slice(),
            ) {
                Ok(path) => path,
                Err(_) => return FfiResult::error(UTF8_DECODE_ERROR).into(),
            })) {
                Ok(file) => file,
                Err(_) => return FfiResult::error(OPEN_FILE_ERROR).into(),
            },
        )
        .into_any()
    })
    .into()
}

#[no_mangle]
extern "C" fn _pen_os_read_file(file: ffi::Any) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    let mut buffer = vec![];
    let result = match unsafe { FfiFile::from_any(file) }.get_mut() {
        Ok(mut file) => file.read_to_end(&mut buffer),
        Err(_) => return FfiResult::error(LOCK_FILE_ERROR).into(),
    };

    match result {
        Ok(_) => FfiResult::ok(buffer.into()),
        Err(_) => FfiResult::error(READ_FILE_ERROR),
    }
    .into()
}

#[no_mangle]
extern "C" fn _pen_os_write_file(
    file: ffi::Any,
    bytes: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Number>> {
    let result = match unsafe { FfiFile::from_any(file) }.get_mut() {
        Ok(mut file) => file.write(bytes.as_slice()),
        Err(_) => return FfiResult::error(LOCK_FILE_ERROR).into(),
    };

    match result {
        Ok(count) => FfiResult::ok(ffi::Number::new(count as f64)),
        Err(_) => FfiResult::error(WRITE_FILE_ERROR),
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_to_any() {
        unsafe { FfiFile::from_any(FfiFile::new(tempfile::tempfile().unwrap()).into_any()) }
            .get_mut()
            .unwrap()
            .write_all(b"foo")
            .unwrap();
    }
}
