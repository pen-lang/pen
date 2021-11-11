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

impl From<Result<(), OsError>> for FfiResult<ffi::None> {
    fn from(result: Result<(), OsError>) -> Self {
        match result {
            Ok(_) => FfiResult::ok(ffi::None::new()),
            Err(error) => FfiResult::error(f64::from(error)),
        }
    }
}

impl<T: Default, E: Into<OsError>> From<Result<T, E>> for FfiResult<T> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(data) => FfiResult::ok(data),
            Err(error) => FfiResult::error(f64::from(error.into())),
        }
    }
}
