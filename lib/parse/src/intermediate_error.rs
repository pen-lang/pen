use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

// An intermediate parse error that is expected to be mapped to a proper error
// type using, for example, `expected` and `unexpected_any` combinators
#[derive(Clone, Debug, Default)]
pub struct IntermediateParseError {}

impl Error for IntermediateParseError {}

impl Display for IntermediateParseError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}
