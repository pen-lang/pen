use super::error::OsError;
use std::error::Error;

#[repr(C)]
pub struct FfiResult<T: Default> {
    value: T,
    error: ffi::ByteString,
}

impl<T: Default> FfiResult<T> {
    pub fn ok(value: T) -> Self {
        Self {
            value,
            error: "".into(),
        }
    }

    pub fn error(error: impl Error) -> Self {
        Self {
            value: Default::default(),
            error: error.to_string().into(),
        }
    }
}

impl From<Result<(), OsError>> for FfiResult<ffi::None> {
    fn from(result: Result<(), OsError>) -> Self {
        match result {
            Ok(_) => Self::ok(ffi::None::new()),
            Err(error) => Self::error(error),
        }
    }
}

impl<T: Default, E: Into<OsError>> From<Result<T, E>> for FfiResult<T> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(data) => Self::ok(data),
            Err(error) => Self::error(error.into()),
        }
    }
}
