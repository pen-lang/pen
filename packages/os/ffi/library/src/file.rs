use super::{error::OsError, open_file_options::OpenFileOptions};
use crate::{result::FfiResult, utilities};
use std::{ops::DerefMut, path::Path, sync::Arc};
use tokio::{
    fs::{self, File, OpenOptions},
    sync::{RwLock, RwLockWriteGuard},
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

    pub async fn lock(&self) -> RwLockWriteGuard<'_, File> {
        TryInto::<&OsFileInner>::try_into(&self.inner)
            .unwrap()
            .get_mut()
            .await
    }
}

#[ffi::any]
#[derive(Clone, Debug)]
pub struct OsFileInner {
    file: Arc<RwLock<File>>,
}

impl OsFileInner {
    pub fn new(file: File) -> Self {
        Self {
            file: Arc::new(RwLock::new(file)),
        }
    }

    pub async fn get_mut(&self) -> RwLockWriteGuard<'_, File> {
        self.file.write().await
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
async fn _pen_os_open_file(
    path: ffi::ByteString,
    options: ffi::Arc<OpenFileOptions>,
) -> ffi::Arc<FfiResult<ffi::Arc<OsFile>>> {
    ffi::Arc::new(open_file(path, options).await.into())
}

async fn open_file(
    path: ffi::ByteString,
    options: ffi::Arc<OpenFileOptions>,
) -> Result<ffi::Arc<OsFile>, OsError> {
    Ok(OsFile::new(
        OpenOptions::from(*options)
            .open(&Path::new(&utilities::decode_path(&path)?))
            .await?,
    ))
}

#[ffi::bindgen]
async fn _pen_os_read_file(file: ffi::Arc<OsFile>) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    ffi::Arc::new(read_file(file).await.into())
}

async fn read_file(file: ffi::Arc<OsFile>) -> Result<ffi::ByteString, OsError> {
    utilities::read(file.lock().await.deref_mut()).await
}

#[ffi::bindgen]
async fn _pen_os_read_limit_file(
    file: ffi::Arc<OsFile>,
    limit: ffi::Number,
) -> ffi::Arc<FfiResult<ffi::ByteString>> {
    ffi::Arc::new(read_limit_file(file, limit).await.into())
}

async fn read_limit_file(
    file: ffi::Arc<OsFile>,
    limit: ffi::Number,
) -> Result<ffi::ByteString, OsError> {
    utilities::read_limit(file.lock().await.deref_mut(), f64::from(limit) as usize).await
}

#[ffi::bindgen]
async fn _pen_os_write_file(
    file: ffi::Arc<OsFile>,
    bytes: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Number>> {
    ffi::Arc::new(write_file(file, bytes).await.into())
}

async fn write_file(
    file: ffi::Arc<OsFile>,
    bytes: ffi::ByteString,
) -> Result<ffi::Number, OsError> {
    utilities::write(file.lock().await.deref_mut(), bytes).await
}

#[ffi::bindgen]
async fn _pen_os_copy_file(
    src: ffi::ByteString,
    dest: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::None>> {
    ffi::Arc::new(copy_file(src, dest).await.into())
}

async fn copy_file(src: ffi::ByteString, dest: ffi::ByteString) -> Result<(), OsError> {
    fs::copy(
        utilities::decode_path(&src)?,
        utilities::decode_path(&dest)?,
    )
    .await?;

    Ok(())
}

#[ffi::bindgen]
async fn _pen_os_move_file(
    src: ffi::ByteString,
    dest: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::None>> {
    ffi::Arc::new(move_file(src, dest).await.into())
}

async fn move_file(src: ffi::ByteString, dest: ffi::ByteString) -> Result<(), OsError> {
    fs::rename(
        utilities::decode_path(&src)?,
        utilities::decode_path(&dest)?,
    )
    .await?;

    Ok(())
}

#[ffi::bindgen]
async fn _pen_os_remove_file(path: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::None>> {
    ffi::Arc::new(remove_file(path).await.into())
}

async fn remove_file(path: ffi::ByteString) -> Result<(), OsError> {
    fs::remove_file(utilities::decode_path(&path)?).await?;

    Ok(())
}

#[ffi::bindgen]
async fn _pen_os_read_metadata(
    path: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Arc<FileMetadata>>> {
    ffi::Arc::new(read_metadata(path).await.into())
}

async fn read_metadata(path: ffi::ByteString) -> Result<ffi::Arc<FileMetadata>, OsError> {
    let metadata = fs::metadata(utilities::decode_path(&path)?).await?;

    Ok(FileMetadata {
        size: (metadata.len() as f64).into(),
    }
    .into())
}
