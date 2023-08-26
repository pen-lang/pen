use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub enum RuntimeError {
    HandleNotInitialized,
    HandleLockPoisoned,
}

impl Display for RuntimeError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::HandleNotInitialized => write!(formatter, "runtime handle not initialized"),
            Self::HandleLockPoisoned => write!(formatter, "handle lock poisoned"),
        }
    }
}

impl Error for RuntimeError {}
