use super::open_file_options::OpenFileOptions;
use crate::utilities;
use std::{error::Error, ops::DerefMut, path::Path, sync::Arc};
use tokio::{
    fs::{self, File, OpenOptions},
    sync::{RwLock, RwLockWriteGuard},
};

extern "C" {
    fn _pen_os_file_to_any(file: ffi::Arc<OsFile>) -> ffi::Any;
}

#[derive(Clone, Default)]
pub struct OsFile {
    inner: ffi::Any,
}

impl OsFile {
    pub fn new(file: File) -> Self {
        Self {
            inner: OsFileInner::new(file).into(),
        }
    }

    pub async fn lock(&self) -> RwLockWriteGuard<'_, File> {
        TryInto::<&OsFileInner>::try_into(&self.inner)
            .unwrap()
            .get_mut()
            .await
    }
}

impl Into<ffi::Any> for OsFile {
    fn into(self) -> ffi::Any {
        unsafe { _pen_os_file_to_any(ffi::Arc::new(self)) }
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
) -> Result<OsFile, Box<dyn Error>> {
    Ok(OsFile::new(
        OpenOptions::from(*options)
            .open(&Path::new(&utilities::decode_path(&path)?))
            .await?,
    ))
}

#[ffi::bindgen]
async fn _pen_os_read_file(file: ffi::Arc<OsFile>) -> Result<ffi::ByteString, Box<dyn Error>> {
    Ok(utilities::read(file.lock().await.deref_mut()).await?)
}

#[ffi::bindgen]
async fn _pen_os_read_limit_file(
    file: ffi::Arc<OsFile>,
    limit: ffi::Number,
) -> Result<ffi::ByteString, Box<dyn Error>> {
    Ok(utilities::read_limit(file.lock().await.deref_mut(), f64::from(limit) as usize).await?)
}

#[ffi::bindgen]
async fn _pen_os_write_file(
    file: ffi::Arc<OsFile>,
    bytes: ffi::ByteString,
) -> Result<ffi::Number, Box<dyn Error>> {
    Ok(utilities::write(file.lock().await.deref_mut(), bytes).await?)
}

#[ffi::bindgen]
async fn _pen_os_copy_file(
    src: ffi::ByteString,
    dest: ffi::ByteString,
) -> Result<(), Box<dyn Error>> {
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
) -> Result<(), Box<dyn Error>> {
    fs::rename(
        utilities::decode_path(&src)?,
        utilities::decode_path(&dest)?,
    )
    .await?;

    Ok(())
}

#[ffi::bindgen]
async fn _pen_os_remove_file(path: ffi::ByteString) -> Result<(), Box<dyn Error>> {
    fs::remove_file(utilities::decode_path(&path)?).await?;

    Ok(())
}

#[ffi::bindgen]
async fn _pen_os_read_metadata(path: ffi::ByteString) -> Result<FileMetadata, Box<dyn Error>> {
    let metadata = fs::metadata(utilities::decode_path(&path)?).await?;

    Ok(FileMetadata {
        size: (metadata.len() as f64).into(),
    })
}
