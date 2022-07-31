use crate::{Any, BoxAny, Error, None};

// This type must be used only as a return type.
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

#[cfg(feature = "std")]
impl<T: Into<Any>> From<core::result::Result<T, alloc::boxed::Box<dyn std::error::Error>>>
    for Result
{
    fn from(result: core::result::Result<T, alloc::boxed::Box<dyn std::error::Error>>) -> Self {
        use alloc::string::ToString;

        Self(BoxAny::from(match result {
            Ok(value) => value.into(),
            Err(error) => Error::new(crate::ByteString::from(error.to_string())).into(),
        }))
    }
}
