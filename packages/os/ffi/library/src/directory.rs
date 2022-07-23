use crate::{error::OsError, utilities};
use tokio::fs;

#[ffi::bindgen]
async fn _pen_os_read_directory(
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
async fn _pen_os_create_directory(path: ffi::ByteString) -> Result<(), OsError> {
    Ok(fs::create_dir(utilities::decode_path(&path)?).await?)
}

#[ffi::bindgen]
async fn _pen_os_remove_directory(path: ffi::ByteString) -> Result<(), OsError> {
    Ok(fs::remove_dir(utilities::decode_path(&path)?).await?)
}
