use crate::{position::Position, types};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    FmmLlvmCompile(fmm_llvm::CompileError),
    FunctionExpected(Position),
    MirFmmCompile(mir_fmm::CompileError),
    TypeNotFound(types::Reference),
    TypeNotInferred(Position),
    TypesNotMatched(Position, Position),
}

impl Display for CompileError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::FmmLlvmCompile(error) => {
                write!(formatter, "failed to compile F-- to LLVM: {}", error)
            }
            Self::FunctionExpected(position) => {
                write!(formatter, "function expected\n{}", position)
            }
            Self::MirFmmCompile(error) => {
                write!(formatter, "failed to compile MIR to F--: {}", error)
            }
            Self::TypeNotFound(reference) => write!(
                formatter,
                "type \"{}\" not found\n{}",
                reference.name(),
                reference.position()
            ),
            Self::TypeNotInferred(position) => {
                write!(formatter, "type not inferred\n{}", position)
            }
            Self::TypesNotMatched(lhs_source_information, rhs_source_information) => write!(
                formatter,
                "types not matched\n{}\n{}",
                lhs_source_information, rhs_source_information
            ),
        }
    }
}

impl Error for CompileError {}

impl From<fmm_llvm::CompileError> for CompileError {
    fn from(error: fmm_llvm::CompileError) -> Self {
        Self::FmmLlvmCompile(error)
    }
}

impl From<mir_fmm::CompileError> for CompileError {
    fn from(error: mir_fmm::CompileError) -> Self {
        Self::MirFmmCompile(error)
    }
}
