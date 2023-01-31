use std::{error::Error, fmt::Display};

#[derive(Clone, Debug)]
pub enum ApplicationError {
    ArchitectureWordSize(String),
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
            Self::ArchitectureWordSize(target_triple) => {
                write!(
                    formatter,
                    "cannot infer word size from target triple: {target_triple}"
                )
            }
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
                write!(formatter, "module {module} not found")
            }
            Self::NewContextFunctionNotFound => {
                write!(formatter, "new context function not found")
            }
            Self::PackageDependencyCycle => {
                write!(formatter, "package dependency cycle detected")
            }
            Self::PackageNotFound(package) => {
                write!(formatter, "package {package} not found")
            }
            Self::SystemPackageNotFound => {
                write!(formatter, "system package not found")
            }
            Self::Test => write!(formatter, "test failed"),
        }
    }
}
