use crate::{ByteString, None};
use alloc::string::ToString;
use core::fmt::Display;

#[repr(C)]
pub struct Result<T> {
    value: T,
    error: ByteString,
}

impl<T> Result<T> {
    pub fn ok(value: T) -> Self {
        Self {
            value,
            error: Default::default(),
        }
    }

    pub fn from_result<E: Display>(
        result: core::result::Result<T, E>,
        value: impl Fn() -> T,
    ) -> Self {
        match result {
            Ok(value) => Self::ok(value),
            Err(error) => Self {
                value: value(),
                error: error.to_string().into(),
            },
        }
    }
}

impl<T: Default> Result<T> {
    pub fn error<E: Display>(error: E) -> Self {
        Self {
            value: Default::default(),
            error: error.to_string().into(),
        }
    }
}

impl<E: Display> From<core::result::Result<(), E>> for Result<None> {
    fn from(result: core::result::Result<(), E>) -> Self {
        result.map(|_| None::default()).into()
    }
}

impl<T: Default, E: Display> From<core::result::Result<T, E>> for Result<T> {
    fn from(result: core::result::Result<T, E>) -> Self {
        Self::from_result(result, || Default::default())
    }
}
