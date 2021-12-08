use crate::{error::OsError, result::FfiResult, utilities};
use tokio::fs;

#[ffi::bindgen]
async fn _pen_os_read_directory(
    path: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Arc<ffi::extra::StringArray>>> {
    ffi::Arc::new(read_directory(path).await.into())
}

async fn read_directory(
    path: ffi::ByteString,
) -> Result<ffi::Arc<ffi::extra::StringArray>, OsError> {
    let mut read_dir = fs::read_dir(utilities::decode_path(&path)?).await?;
    let mut entries = vec![];

    while let Some(entry) = read_dir.next_entry().await? {
        entries.push(entry);
    }

    Ok(ffi::Arc::new(
        entries
            .into_iter()
            .map(|entry| {
                Ok(ffi::ByteString::from(
                    entry
                        .file_name()
                        .into_string()
                        .map_err(|_| OsError::Other("cannot decode path".into()))?,
                ))
            })
            .collect::<Result<Vec<_>, OsError>>()?
            .into(),
    ))
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
