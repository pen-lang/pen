use super::{error::OsError, open_file_options::OpenFileOptions, utilities};
use crate::result::FfiResult;
use ffi::AnyLike;
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
            inner: OsFileInner::new(file).into_any(),
        })
    }

    pub fn lock(&self) -> Result<RwLockWriteGuard<File>, OsError> {
        Ok(OsFileInner::as_inner(&self.inner).unwrap().get_mut()?)
    }
}

#[derive(Clone, Debug)]
pub struct OsFileInner {
    file: Arc<RwLock<File>>,
}

ffi::type_information!(ffi_file, crate::file::OsFileInner);

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

#[no_mangle]
extern "C" fn _pen_os_open_file(
    path: ffi::ByteString,
    options: ffi::Arc<OpenFileOptions>,
) -> ffi::Arc<FfiResult<ffi::Arc<OsFile>>> {
    ffi::Arc::new(open_file(path, options).into())
}

fn open_file(
    path: ffi::ByteString,
    options: ffi::Arc<OpenFileOptions>,
) -> Result<ffi::Arc<OsFile>, OsError> {
    Ok(OsFile::new(
        OpenOptions::from(options.deref()).open(&Path::new(&utilities::decode_path(&path)?))?,
    ))
}

#[no_mangle]
extern "C" fn _pen_os_read_file(file: ffi::Arc<OsFile>) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    ffi::Arc::new(read_file(file).into())
}

fn read_file(file: ffi::Arc<OsFile>) -> Result<ffi::ByteString, OsError> {
    utilities::read(&mut file.lock()?.deref())
}

#[no_mangle]
extern "C" fn _pen_os_write_file(
    file: ffi::Arc<OsFile>,
    bytes: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(write_file(file, bytes).into())
}

fn write_file(file: ffi::Arc<OsFile>, bytes: ffi::ByteString) -> Result<ffi::Number, OsError> {
    utilities::write(&mut file.lock()?.deref(), bytes)
}

#[no_mangle]
extern "C" fn _pen_os_copy_file(
    src: ffi::ByteString,
    dest: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::None>> {
    ffi::Arc::new(copy_file(src, dest).into())
}

fn copy_file(src: ffi::ByteString, dest: ffi::ByteString) -> Result<(), OsError> {
    fs::copy(
        utilities::decode_path(&src)?,
        utilities::decode_path(&dest)?,
    )?;

    Ok(())
}

#[no_mangle]
extern "C" fn _pen_os_remove_file(path: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::None>> {
    ffi::Arc::new(remove_file(path).into())
}

fn remove_file(path: ffi::ByteString) -> Result<(), OsError> {
    fs::remove_file(utilities::decode_path(&path)?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn convert_to_any() {
        OsFileInner::from_any(OsFileInner::new(tempfile::tempfile().unwrap()).into_any())
            .unwrap()
            .get_mut()
            .unwrap()
            .write_all(b"foo")
            .unwrap();
    }
}
