use super::type_information;
use crate::any::Any;
use crate::result::FfiResult;
use std::fs::File;
use std::path::Path;
use std::str;
use std::sync::Arc;
use std::sync::LockResult;
use std::sync::RwLock;
use std::sync::RwLockWriteGuard;

const UTF8_DECODE_ERROR: f64 = 0.0;
const OPEN_FILE_ERROR: f64 = 1.0;

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
        Ok(self.file.write()?)
    }
}

impl From<File> for FfiFile {
    fn from(file: File) -> Self {
        Self::new(file)
    }
}

#[no_mangle]
extern "C" fn _pen_os_open_file(path: ffi::ByteString) -> ffi::Arc<FfiResult<Any>> {
    FfiResult::ok(
        FfiFile::new(
            match File::open(&Path::new(&match str::from_utf8(path.as_slice()) {
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
