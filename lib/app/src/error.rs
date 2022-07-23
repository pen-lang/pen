use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ApplicationError {
    Build,
    ContextTypeNotFound,
    ModuleFilesNotFormatted(Vec<String>),
    ModuleNotFound(String),
    NewContextFunctionNotFound,
    PackageDependencyCycle,
    PackageNotFound(String),
    SystemPackageNotFound,
    Test,
}

impl Error for ApplicationError {}

impl Display for ApplicationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::Build => write!(formatter, "build failed"),
            Self::ContextTypeNotFound => {
                write!(formatter, "context type not found")
            }
            Self::ModuleFilesNotFormatted(paths) => {
                write!(
                    formatter,
                    "module files not formatted: {}",
                    paths.join(", ")
                )
            }
            Self::ModuleNotFound(module) => {
                write!(formatter, "module {} not found", module)
            }
            Self::NewContextFunctionNotFound => {
                write!(formatter, "new context function not found")
            }
            Self::PackageDependencyCycle => {
                write!(formatter, "package dependency cycle detected")
            }
            Self::PackageNotFound(package) => {
                write!(formatter, "package {} not found", package)
            }
            Self::SystemPackageNotFound => {
                write!(formatter, "system package not found")
            }
            Self::Test => write!(formatter, "test failed"),
        }
    }
}
