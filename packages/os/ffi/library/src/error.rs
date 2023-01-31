use std::{
    env::VarError,
    error::Error,
    fmt::{self, Display, Formatter},
    io,
    str::Utf8Error,
    sync::PoisonError,
};

#[derive(Debug)]
pub enum OsError {
    EnvironmentVariable(VarError),
    Io(io::Error),
    Utf8(Utf8Error),
    Other(String),
}

impl Display for OsError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::EnvironmentVariable(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Utf8(error) => write!(formatter, "{error}"),
            Self::Other(message) => write!(formatter, "{}", &message),
        }
    }
}

impl Error for OsError {}

impl From<io::Error> for OsError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl<T> From<PoisonError<T>> for OsError {
    fn from(error: PoisonError<T>) -> Self {
        Self::Other(error.to_string())
    }
}

impl From<Utf8Error> for OsError {
    fn from(error: Utf8Error) -> Self {
        Self::Utf8(error)
    }
}

impl From<VarError> for OsError {
    fn from(error: VarError) -> Self {
        Self::EnvironmentVariable(error)
    }
}
