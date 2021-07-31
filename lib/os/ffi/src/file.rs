use super::open_file_options::OpenFileOptions;
use super::type_information;
use crate::{any::Any, result::FfiResult};
use std::ops::Deref;
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
    str,
    sync::{Arc, LockResult, RwLock, RwLockWriteGuard},
};

const UTF8_DECODE_ERROR: f64 = 0.0;
const OPEN_FILE_ERROR: f64 = 1.0;
const LOCK_FILE_ERROR: f64 = 2.0;
const WRITE_FILE_ERROR: f64 = 3.0;

#[derive(Clone, Debug)]
pub struct FfiFile {
    file: Arc<RwLock<File>>,
}

type_information!(ffi_file, crate::file::FfiFile);

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
) -> ffi::Arc<FfiResult<Any>> {
    FfiResult::ok(
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
        .into(),
    )
    .into()
}

#[no_mangle]
extern "C" fn _pen_os_write_file(
    file: Any,
    bytes: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::None>> {
    let result = match FfiFile::from(file).get_mut() {
        Ok(mut file) => file.write(bytes.as_slice()),
        Err(_) => return FfiResult::error(LOCK_FILE_ERROR).into(),
    };

    if let Err(_) = result {
        return FfiResult::error(WRITE_FILE_ERROR).into();
    }

    FfiResult::ok(ffi::None::new()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_to_any() {
        FfiFile::from(Any::from(FfiFile::new(tempfile::tempfile().unwrap())))
            .get_mut()
            .unwrap()
            .write(b"foo")
            .unwrap();
    }
}
