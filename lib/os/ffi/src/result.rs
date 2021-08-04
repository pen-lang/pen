use super::error::OsError;

#[repr(C)]
pub struct FfiResult<T: Default> {
    value: T,
    error: ffi::Number,
}

impl<T: Default> FfiResult<T> {
    pub fn ok(value: T) -> Self {
        Self {
            value,
            error: (0.0).into(),
        }
    }

    pub fn error(error: impl Into<ffi::Number>) -> Self {
        Self {
            value: Default::default(),
            error: error.into(),
        }
    }
}

impl<T: Default> From<Result<T, OsError>> for FfiResult<T> {
    fn from(result: Result<T, OsError>) -> Self {
        match result {
            Ok(data) => FfiResult::ok(data).into(),
            Err(error) => FfiResult::error(f64::from(error)),
        }
    }
}
