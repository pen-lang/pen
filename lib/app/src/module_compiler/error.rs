use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ModuleCompilerError {
    MainFunctionTypeNotFound,
}

impl Error for ModuleCompilerError {}

impl Display for ModuleCompilerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::MainFunctionTypeNotFound => {
                write!(formatter, "main function type not found")
            }
        }
    }
}
