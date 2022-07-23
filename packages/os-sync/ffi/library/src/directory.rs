use crate::{error::OsError, utilities};
use std::fs;

#[ffi::bindgen]
fn _pen_os_read_directory(
    path: ffi::ByteString,
) -> Result<ffi::Arc<ffi::extra::StringArray>, OsError> {
    Ok(ffi::Arc::new(
        fs::read_dir(utilities::decode_path(&path)?)?
            .collect::<Result<Vec<_>, _>>()?
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
fn _pen_os_create_directory(path: ffi::ByteString) -> Result<(), OsError> {
    Ok(fs::create_dir(utilities::decode_path(&path)?)?)
}

#[ffi::bindgen]
fn _pen_os_remove_directory(path: ffi::ByteString) -> Result<(), OsError> {
    Ok(fs::remove_dir(utilities::decode_path(&path)?)?)
}
