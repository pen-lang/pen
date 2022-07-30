use crate::{Any, BoxAny, Error, None};

#[repr(transparent)]
pub struct Result(BoxAny);

impl<E: Into<Any>> From<core::result::Result<(), E>> for Result {
    fn from(result: core::result::Result<(), E>) -> Self {
        result.map(|_| None::default()).into()
    }
}

impl<T: Into<Any>, E: Into<Any>> From<core::result::Result<T, E>> for Result {
    fn from(result: core::result::Result<T, E>) -> Self {
        Self(BoxAny::from(match result {
            Ok(value) => value.into(),
            Err(error) => Error::new(error.into()).into(),
        }))
    }
}
