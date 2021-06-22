use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ModuleDependencyCompilerError {
    PackageNotFound(String),
}

impl Error for ModuleDependencyCompilerError {}

impl Display for ModuleDependencyCompilerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::PackageNotFound(package) => {
                write!(formatter, "package {} not found", package)
            }
        }
    }
}
