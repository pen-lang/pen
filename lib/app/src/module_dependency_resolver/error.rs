use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ModuleDependencyResolverError {
    PackageNotFound(String),
}

impl Error for ModuleDependencyResolverError {}

impl Display for ModuleDependencyResolverError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::PackageNotFound(package) => {
                write!(formatter, "package {} not found", package)
            }
        }
    }
}
