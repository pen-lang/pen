use core::fmt::{self, Display, Formatter};
use std::error::Error;

#[derive(Debug, Clone)]
pub enum RuntimeError {
    HandleNotInitialized,
    HandleLockPoisoned,
}

impl Display for RuntimeError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::HandleNotInitialized => write!(formatter, "runtime handle not initialized"),
            Self::HandleLockPoisoned => write!(formatter, "runtime handle lock poisoned"),
        }
    }
}

impl Error for RuntimeError {}
