use position::Position;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    num::{ParseFloatError, ParseIntError},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompileError {
    ModuleNotFound(ast::ModulePath),
    NameNotFound(String, Position),
    ParseFloat {
        error: ParseFloatError,
        position: Position,
    },
    ParseInteger {
        error: ParseIntError,
        position: Position,
    },
    TooFewBranchesInIf(Position),
}

impl Display for CompileError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::ModuleNotFound(path) => {
                write!(formatter, "module \"{path}\" not found")
            }
            Self::NameNotFound(name, position) => {
                write!(formatter, "name \"{name}\" not found\n{position}")
            }
            Self::ParseFloat { error, position } => {
                write!(formatter, "{error}\n{position}")
            }
            Self::ParseInteger { error, position } => {
                write!(formatter, "{error}\n{position}")
            }
            Self::TooFewBranchesInIf(position) => {
                write!(formatter, "too small number of branches in if\n{position}")
            }
        }
    }
}

impl Error for CompileError {}
