use std::{env::VarError, str::Utf8Error, sync::PoisonError};

#[derive(Debug)]
pub enum OsError {
    EnvironmentVariableNotPresent,
    EnvironmentVariableNotUnicode,
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
            OsError::EnvironmentVariableNotPresent => 259.0,
            OsError::EnvironmentVariableNotUnicode => 260.0,
            OsError::Unknown => 512.0,
        }
    }
}

impl<T> From<PoisonError<T>> for OsError {
    fn from(_: PoisonError<T>) -> Self {
        Self::LockFile
    }
}

impl From<std::io::Error> for OsError {
    fn from(error: std::io::Error) -> Self {
        if let Some(code) = error.raw_os_error() {
            Self::Raw(code)
        } else {
            Self::Unknown
        }
    }
}

impl From<Utf8Error> for OsError {
    fn from(_: Utf8Error) -> Self {
        Self::Utf8Decode
    }
}

impl From<VarError> for OsError {
    fn from(error: VarError) -> Self {
        match error {
            VarError::NotPresent => Self::EnvironmentVariableNotPresent,
            VarError::NotUnicode(_) => Self::EnvironmentVariableNotUnicode,
        }
    }
}
