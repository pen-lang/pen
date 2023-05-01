use super::open_file_options::OpenFileOptions;
use crate::utilities;
use std::{error::Error, ops::DerefMut, path::Path, sync::Arc};
use tokio::{
    fs,
    sync::{RwLock, RwLockWriteGuard},
};

#[ffi::into_any(into_fn = "_pen_os_file_to_any")]
#[repr(C)]
#[derive(Clone)]
struct File(ffi::Arc<ffi::Any>);

#[ffi::any]
#[derive(Clone)]
struct FileInner(Arc<RwLock<fs::File>>);

impl File {
    pub fn new(file: fs::File) -> Self {
        Self(ffi::Arc::new(FileInner(Arc::new(RwLock::new(file))).into()))
    }

    pub async fn lock(&self) -> RwLockWriteGuard<'_, fs::File> {
        TryInto::<&FileInner>::try_into(&*self.0)
            .unwrap()
            .0
            .write()
            .await
    }
}

#[ffi::into_any(into_fn = "_pen_os_file_metadata_to_any")]
#[repr(C)]
struct FileMetadata(ffi::Arc<FileMetadataInner>);

#[repr(C)]
struct FileMetadataInner {
    size: ffi::Number,
}

impl FileMetadata {
    pub fn new(size: ffi::Number) -> Self {
        Self(FileMetadataInner { size }.into())
    }
}

#[ffi::bindgen]
async fn _pen_os_open_file(
    path: ffi::ByteString,
    options: OpenFileOptions,
) -> Result<File, Box<dyn Error>> {
    Ok(File::new(
        fs::OpenOptions::from(options)
            .open(&Path::new(&utilities::decode_path(&path)?))
            .await?,
    ))
}

#[ffi::bindgen]
async fn _pen_os_read_file(file: File) -> Result<ffi::ByteString, Box<dyn Error>> {
    utilities::read(file.lock().await.deref_mut()).await
}

#[ffi::bindgen]
async fn _pen_os_read_limit_file(
    file: File,
    limit: ffi::Number,
) -> Result<ffi::ByteString, Box<dyn Error>> {
    utilities::read_limit(file.lock().await.deref_mut(), f64::from(limit) as usize).await
}

#[ffi::bindgen]
async fn _pen_os_write_file(
    file: File,
    bytes: ffi::ByteString,
) -> Result<ffi::Number, Box<dyn Error>> {
    utilities::write(file.lock().await.deref_mut(), bytes).await
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

    Ok(FileMetadata::new((metadata.len() as f64).into()))
}
