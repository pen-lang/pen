use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

use super::type_inference::TypeInferenceError;

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    FmmLlvmCompile(fmm_llvm::CompileError),
    MirFmmCompile(mir_fmm::CompileError),
    TypeInference(TypeInferenceError),
}

impl Display for CompileError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::FmmLlvmCompile(error) => {
                write!(formatter, "failed to compile F-- to LLVM: {}", error)
            }
            Self::MirFmmCompile(error) => {
                write!(formatter, "failed to compile MIR to F--: {}", error)
            }
            Self::TypeInference(error) => {
                write!(formatter, "{}", error)
            }
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

impl From<TypeInferenceError> for CompileError {
    fn from(error: TypeInferenceError) -> Self {
        Self::TypeInference(error)
    }
}
