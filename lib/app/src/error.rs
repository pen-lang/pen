use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ApplicationError {
    Build,
    MainFunctionTypeNotFound,
    ModuleNotFound(String),
    PackageNotFound(String),
    SystemPackageNotFound,
    TooManySystemPackages,
    Test,
}

impl Error for ApplicationError {}

impl Display for ApplicationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::Build => write!(formatter, "build failed"),
            Self::MainFunctionTypeNotFound => {
                write!(formatter, "main function type not found")
            }
            Self::ModuleNotFound(module) => {
                write!(formatter, "module {} not found", module)
            }
            Self::PackageNotFound(package) => {
                write!(formatter, "package {} not found", package)
            }
            Self::SystemPackageNotFound => {
                write!(formatter, "system package not found")
            }
            Self::TooManySystemPackages => {
                write!(formatter, "too many system package")
            }
            Self::Test => write!(formatter, "test failed"),
        }
    }
}
