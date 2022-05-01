use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug)]
pub enum SqlError {
    TypeNotSupported,
}

impl Display for SqlError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            SqlError::TypeNotSupported => write!(formatter, "type not supported"),
        }
    }
}

impl Error for SqlError {}
