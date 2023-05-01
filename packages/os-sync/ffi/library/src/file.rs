use super::{error::OsError, open_file_options::OpenFileOptions, utilities};
use std::{
    error::Error,
    fs,
    ops::Deref,
    path::Path,
    sync::{Arc, RwLock, RwLockWriteGuard},
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

    pub fn lock(&self) -> Result<RwLockWriteGuard<fs::File>, OsError> {
        Ok(TryInto::<&FileInner>::try_into(&*self.0)
            .unwrap()
            .0
            .write()?)
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
fn _pen_os_open_file(
    path: ffi::ByteString,
    options: OpenFileOptions,
) -> Result<File, Box<dyn Error>> {
    Ok(File::new(
        fs::OpenOptions::from(options).open(Path::new(&utilities::decode_path(&path)?))?,
    ))
}

#[ffi::bindgen]
fn _pen_os_read_file(file: File) -> Result<ffi::ByteString, Box<dyn Error>> {
    utilities::read(&mut file.lock()?.deref())
}

#[ffi::bindgen]
fn _pen_os_read_limit_file(
    file: File,
    limit: ffi::Number,
) -> Result<ffi::ByteString, Box<dyn Error>> {
    utilities::read_limit(&mut file.lock()?.deref(), f64::from(limit) as usize)
}

#[ffi::bindgen]
fn _pen_os_write_file(file: File, bytes: ffi::ByteString) -> Result<ffi::Number, Box<dyn Error>> {
    utilities::write(&mut file.lock()?.deref(), bytes)
}

#[ffi::bindgen]
fn _pen_os_copy_file(src: ffi::ByteString, dest: ffi::ByteString) -> Result<(), Box<dyn Error>> {
    fs::copy(
        utilities::decode_path(&src)?,
        utilities::decode_path(&dest)?,
    )?;

    Ok(())
}

#[ffi::bindgen]
fn _pen_os_move_file(src: ffi::ByteString, dest: ffi::ByteString) -> Result<(), Box<dyn Error>> {
    fs::rename(
        utilities::decode_path(&src)?,
        utilities::decode_path(&dest)?,
    )?;

    Ok(())
}

#[ffi::bindgen]
fn _pen_os_remove_file(path: ffi::ByteString) -> Result<(), Box<dyn Error>> {
    fs::remove_file(utilities::decode_path(&path)?)?;

    Ok(())
}

#[ffi::bindgen]
fn _pen_os_read_metadata(path: ffi::ByteString) -> Result<FileMetadata, Box<dyn Error>> {
    let metadata = fs::metadata(utilities::decode_path(&path)?)?;

    Ok(FileMetadata::new((metadata.len() as f64).into()))
}
