use std::mem::MaybeUninit;

#[repr(C)]
pub struct FfiResult<T> {
    value: T,
    error: ffi::Number,
}

impl<T> FfiResult<T> {
    pub fn ok(value: T) -> Self {
        Self {
            value,
            error: (-1.0).into(),
        }
    }

    pub fn error(error: impl Into<ffi::Number>) -> Self {
        Self {
            value: unsafe { MaybeUninit::uninit().assume_init() },
            error: error.into(),
        }
    }
}

impl<T: Default> From<std::io::Error> for FfiResult<T> {
    fn from(error: std::io::Error) -> Self {
        Self::error(error.raw_os_error().map(f64::from).unwrap_or(std::f64::NAN))
    }
}
