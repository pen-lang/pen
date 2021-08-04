use std::{str::Utf8Error, sync::PoisonError};

pub enum OsError {
    Raw(i32),
    LockFile,
    Utf8Decode,
    Unknown,
}

impl From<OsError> for f64 {
    fn from(error: OsError) -> Self {
        match error {
            OsError::Raw(code) => code.into(),
            OsError::LockFile => 257.0,
            OsError::Utf8Decode => 258.0,
            OsError::Unknown => 512.0,
        }
    }
}

impl<T> From<PoisonError<T>> for OsError {
    fn from(_: PoisonError<T>) -> OsError {
        OsError::LockFile
    }
}

impl From<std::io::Error> for OsError {
    fn from(error: std::io::Error) -> Self {
        if let Some(code) = error.raw_os_error() {
            OsError::Raw(code)
        } else {
            OsError::Unknown
        }
    }
}

impl From<Utf8Error> for OsError {
    fn from(_: std::str::Utf8Error) -> OsError {
        OsError::Utf8Decode
    }
}
