use super::{error::*, open_file_options::OpenFileOptions, utilities};
use crate::result::FfiResult;
use std::fs;
use std::{
    fs::{File, OpenOptions},
    ops::Deref,
    path::Path,
    str,
    sync::{Arc, LockResult, RwLock, RwLockWriteGuard},
};

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
    ffi::Arc::new(open_file(path, options).into())
}

fn open_file(path: ffi::ByteString, options: ffi::Arc<OpenFileOptions>) -> Result<ffi::Any, f64> {
    let file = FfiFile::new(
        OpenOptions::from(options.deref())
            .open(&Path::new(&decode_path(&path)?))
            .map_err(|_| OPEN_FILE_ERROR)?,
    );

    Ok(unsafe { file.into_any() })
}

#[no_mangle]
extern "C" fn _pen_os_read_file(file: ffi::Any) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    ffi::Arc::new(read_file(file).into())
}

fn read_file(file: ffi::Any) -> Result<ffi::ByteString, f64> {
    utilities::read(&mut lock_file(&unsafe { FfiFile::from_any(file) })?.deref())
}

#[no_mangle]
extern "C" fn _pen_os_write_file(
    file: ffi::Any,
    bytes: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(write_file(file, bytes).into())
}

fn write_file(file: ffi::Any, bytes: ffi::ByteString) -> Result<ffi::Number, f64> {
    utilities::write(
        &mut lock_file(&unsafe { FfiFile::from_any(file) })?.deref(),
        bytes,
    )
}

fn lock_file(file: &FfiFile) -> Result<RwLockWriteGuard<File>, f64> {
    Ok(file.get_mut().map_err(|_| LOCK_FILE_ERROR)?)
}

#[no_mangle]
extern "C" fn _pen_os_copy_file(
    src: ffi::ByteString,
    dest: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::None>> {
    ffi::Arc::new(copy_file(src, dest).into())
}

fn copy_file(src: ffi::ByteString, dest: ffi::ByteString) -> Result<ffi::None, f64> {
    fs::copy(decode_path(&src)?, decode_path(&dest)?).map_err(|_| COPY_FILE_ERROR)?;

    Ok(ffi::None::new())
}

#[no_mangle]
extern "C" fn _pen_os_remove_file(path: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::None>> {
    ffi::Arc::new(remove_file(path).into())
}

fn remove_file(path: ffi::ByteString) -> Result<ffi::None, f64> {
    fs::remove_file(decode_path(&path)?).map_err(|_| REMOVE_FILE_ERROR)?;

    Ok(ffi::None::new())
}

fn decode_path(path: &ffi::ByteString) -> Result<&str, f64> {
    str::from_utf8(path.as_slice()).map_err(|_| DECODE_PATH_ERROR)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn convert_to_any() {
        unsafe { FfiFile::from_any(FfiFile::new(tempfile::tempfile().unwrap()).into_any()) }
            .get_mut()
            .unwrap()
            .write_all(b"foo")
            .unwrap();
    }
}
