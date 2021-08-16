use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ApplicationError {
    MainFunctionTypeNotFound,
    PackageNotFound(String),
    SystemPackageNotFound,
}

impl Error for ApplicationError {}

impl Display for ApplicationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::MainFunctionTypeNotFound => {
                write!(formatter, "main function type not found")
            }
            Self::PackageNotFound(package) => {
                write!(formatter, "package {} not found", package)
            }
            Self::SystemPackageNotFound => {
                write!(formatter, "system package not found")
            }
        }
    }
}
