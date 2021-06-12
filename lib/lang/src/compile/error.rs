use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    MirFmmCompileError(mir_fmm::CompileError),
    FmmLlvmCompileError(fmm_llvm::CompileError),
}

impl Display for CompileError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::MirFmmCompileError(error) => {
                write!(formatter, "failed to compile MIR to F--: {}", error)
            }
            Self::FmmLlvmCompileError(error) => {
                write!(formatter, "failed to compile F-- to LLVM: {}", error)
            }
        }
    }
}

impl Error for CompileError {}

impl From<mir_fmm::CompileError> for CompileError {
    fn from(error: mir_fmm::CompileError) -> Self {
        Self::MirFmmCompileError(error)
    }
}

impl From<fmm_llvm::CompileError> for CompileError {
    fn from(error: fmm_llvm::CompileError) -> Self {
        Self::FmmLlvmCompileError(error)
    }
}
