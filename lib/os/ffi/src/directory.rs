use crate::{array::Array, error::OsError, result::FfiResult};
use std::{fs, str};

#[no_mangle]
extern "C" fn _pen_os_read_directory(
    path: ffi::ByteString,
) -> ffi::Arc<FfiResult<ffi::Arc<Array>>> {
    ffi::Arc::new(read_directory(path).into())
}

fn read_directory(path: ffi::ByteString) -> Result<ffi::Arc<Array>, OsError> {
    Ok(ffi::Arc::new(
        fs::read_dir(decode_path(&path)?)?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|entry| {
                Ok(ffi::ByteString::from(
                    entry
                        .file_name()
                        .into_string()
                        .map_err(|_| OsError::Utf8Decode)?,
                ))
            })
            .collect::<Result<Vec<_>, OsError>>()?
            .into(),
    ))
}

fn decode_path(path: &ffi::ByteString) -> Result<&str, OsError> {
    Ok(str::from_utf8(path.as_slice())?)
}
