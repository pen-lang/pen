use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ApplicationError {
    SystemPackageNotFound,
}

impl Error for ApplicationError {}

impl Display for ApplicationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::SystemPackageNotFound => {
                write!(formatter, "system package not found")
            }
        }
    }
}
