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

#[ffi::bindgen]
fn _pen_os_open_file(
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
        OpenOptions::from(*options).open(&Path::new(&utilities::decode_path(&path)?))?,
    ))
}

#[ffi::bindgen]
fn _pen_os_read_file(file: ffi::Arc<OsFile>) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    ffi::Arc::new(read_file(file).into())
}

fn read_file(file: ffi::Arc<OsFile>) -> Result<ffi::ByteString, OsError> {
    utilities::read(&mut file.lock()?.deref())
}

#[ffi::bindgen]
fn _pen_os_read_limit_file(
    file: ffi::Arc<OsFile>,
    limit: ffi::Number,
) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    ffi::Arc::new(read_limit_file(file, limit).into())
}

fn read_limit_file(file: ffi::Arc<OsFile>, limit: ffi::Number) -> Result<ffi::ByteString, OsError> {
    utilities::read_limit(&mut file.lock()?.deref(), f64::from(limit) as usize)
}

#[ffi::bindgen]
fn _pen_os_write_file(
    file: ffi::Arc<OsFile>,
    bytes: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(write_file(file, bytes).into())
}

fn write_file(file: ffi::Arc<OsFile>, bytes: ffi::ByteString) -> Result<ffi::Number, OsError> {
    utilities::write(&mut file.lock()?.deref(), bytes)
}

#[ffi::bindgen]
fn _pen_os_copy_file(
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

#[ffi::bindgen]
fn _pen_os_move_file(
    src: ffi::ByteString,
    dest: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::None>> {
    ffi::Arc::new(move_file(src, dest).into())
}

fn move_file(src: ffi::ByteString, dest: ffi::ByteString) -> Result<(), OsError> {
    fs::rename(
        utilities::decode_path(&src)?,
        utilities::decode_path(&dest)?,
    )?;

    Ok(())
}

#[ffi::bindgen]
fn _pen_os_remove_file(path: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::None>> {
    ffi::Arc::new(remove_file(path).into())
}

fn remove_file(path: ffi::ByteString) -> Result<(), OsError> {
    fs::remove_file(utilities::decode_path(&path)?)?;

    Ok(())
}

#[ffi::bindgen]
fn _pen_os_read_metadata(path: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::Arc<FileMetadata>>> {
    ffi::Arc::new(read_metadata(path).into())
}

fn read_metadata(path: ffi::ByteString) -> Result<ffi::Arc<FileMetadata>, OsError> {
    let metadata = fs::metadata(utilities::decode_path(&path)?)?;

    Ok(FileMetadata {
        size: (metadata.len() as f64).into(),
    }
    .into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn convert_to_any() {
        OsFileInner::try_from(ffi::Any::from(OsFileInner::new(
            tempfile::tempfile().unwrap(),
        )))
        .unwrap()
        .get_mut()
        .unwrap()
        .write_all(b"foo")
        .unwrap();
    }
}
