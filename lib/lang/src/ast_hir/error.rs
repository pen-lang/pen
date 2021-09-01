use position::Position;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    TooFewBranchesInIf(Position),
}

impl Display for CompileError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::TooFewBranchesInIf(position) => {
                write!(
                    formatter,
                    "too small number of branches in if\n{}",
                    position
                )
            }
        }
    }
}

impl Error for CompileError {}
