use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum PackageBuildScriptCompilerError {
    SystemPackageNotFound,
}

impl Error for PackageBuildScriptCompilerError {}

impl Display for PackageBuildScriptCompilerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::SystemPackageNotFound => {
                write!(formatter, "system package not found")
            }
        }
    }
}
