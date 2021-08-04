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

impl<T: Default> From<std::io::Error> for FfiResult<T> {
    fn from(error: std::io::Error) -> Self {
        Self::error(error.raw_os_error().map(f64::from).unwrap_or(std::f64::NAN))
    }
}

impl<T: Default> From<Result<T, f64>> for FfiResult<T> {
    fn from(result: Result<T, f64>) -> Self {
        match result {
            Ok(data) => FfiResult::ok(data).into(),
            Err(error) => FfiResult::error(error),
        }
    }
}
