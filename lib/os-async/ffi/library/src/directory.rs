use crate::{error::OsError, result::FfiResult, utilities};
use tokio::fs;

#[ffi::bindgen]
async fn _pen_os_read_directory(
    _path: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Arc<ffi::extra::StringArray>>> {
    todo!()
}

#[ffi::bindgen]
async fn _pen_os_create_directory(path: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::None>> {
    ffi::Arc::new(create_directory(path).await.into())
}

async fn create_directory(path: ffi::ByteString) -> Result<(), OsError> {
    Ok(fs::create_dir(utilities::decode_path(&path)?).await?)
}

#[ffi::bindgen]
async fn _pen_os_remove_directory(path: ffi::ByteString) -> ffi::Arc<FfiResult<ffi::None>> {
    ffi::Arc::new(remove_directory(path).await.into())
}

async fn remove_directory(path: ffi::ByteString) -> Result<(), OsError> {
    Ok(fs::remove_dir(utilities::decode_path(&path)?).await?)
}
