use super::{error::OsError, open_file_options::OpenFileOptions, utilities};
use crate::result::FfiResult;
use std::{
    fs,
    fs::{File, OpenOptions},
    ops::Deref,
    path::Path,
    sync::{Arc, LockResult, RwLock, RwLockWriteGuard},
};

#[derive(Clone, Default)]
pub struct OsFile {
    inner: ffi::Any,
}

impl OsFile {
    pub fn new(file: File) -> ffi::Arc<Self> {
        ffi::Arc::new(Self {
            inner: OsFileInner::new(file).into(),
        })
    }

    pub fn lock(&self) -> Result<RwLockWriteGuard<File>, OsError> {
        Ok(TryInto::<&OsFileInner>::try_into(&self.inner)
            .unwrap()
            .get_mut()?)
    }
}

#[derive(Clone, Debug)]
pub struct OsFileInner {
    file: Arc<RwLock<File>>,
}

ffi::type_information!(os_file_inner, crate::file::OsFileInner);

impl OsFileInner {
    pub fn new(file: File) -> Self {
        Self {
            file: Arc::new(RwLock::new(file)),
        }
    }

    pub fn get_mut(&self) -> LockResult<RwLockWriteGuard<'_, File>> {
        self.file.write()
    }
}

impl From<File> for OsFileInner {
    fn from(file: File) -> Self {
        Self::new(file)
    }
}

#[derive(Default)]
#[repr(C)]
struct FileMetadata {
    size: ffi::Number,
}

#[no_mangle]
extern "C" fn _pen_os_open_file(
    path: ffi::ByteString,
    options: ffi::Arc<OpenFileOptions>,
) -> ffi::Arc<FfiResult<ffi::Arc<OsFile>>> {
    todo!()
}

#[no_mangle]
extern "C" fn _pen_os_read_file(file: ffi::Arc<OsFile>) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    todo!()
}

#[no_mangle]
extern "C" fn _pen_os_read_limit_file(
    file: ffi::Arc<OsFile>,
    limit: ffi::Number,
) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    todo!()
}

#[no_mangle]
extern "C" fn _pen_os_write_file(
    file: ffi::Arc<OsFile>,
    bytes: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Number>> {
    todo!()
}

#[no_mangle]
extern "C" fn _pen_os_copy_file(
    src: ffi::ByteString,
    dest: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::None>> {
    todo!()
}

#[no_mangle]
extern "C" fn _pen_os_move_file(
    src: ffi::ByteString,
    dest: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::None>> {
    todo!()
}

#[no_mangle]
extern "C" fn _pen_os_remove_file(path: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::None>> {
    todo!()
}

#[no_mangle]
extern "C" fn _pen_os_read_metadata(
    path: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Arc<FileMetadata>>> {
    todo!()
}
