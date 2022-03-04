use hir::analysis::AnalysisError;
use position::Position;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    Analysis(AnalysisError),
    AnyEqualOperation(Position),
    CompileConfigurationNotProvided,
    FunctionEqualOperation(Position),
    FunctionExpected(Position),
    InvalidRecordEqualOperation(Position),
    MainFunctionNotFound(Position),
    MirTypeCheck(mir::analysis::TypeCheckError),
    NewContextFunctionNotFound(Position),
    VariantTypeInFfi(Position),
}

impl Display for CompileError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Analysis(error) => write!(formatter, "{}", error),
            Self::AnyEqualOperation(position) => {
                write!(
                    formatter,
                    "equal operator cannot be used with any type\n{}",
                    position
                )
            }
            Self::CompileConfigurationNotProvided => {
                write!(formatter, "compile configuration not provided")
            }
            Self::FunctionEqualOperation(position) => {
                write!(
                    formatter,
                    "equal operator cannot be used with function type\n{}",
                    position
                )
            }
            Self::FunctionExpected(position) => {
                write!(formatter, "function expected\n{}", position)
            }
            Self::InvalidRecordEqualOperation(position) => {
                write!(
                    formatter,
                    "equal operator cannot be used with record type containing any or function types\n{}",
                    position
                )
            }
            Self::MainFunctionNotFound(position) => {
                write!(formatter, "main function not found\n{}", position)
            }
            Self::MirTypeCheck(error) => {
                write!(formatter, "failed to check types in MIR: {}", error)
            }
            Self::NewContextFunctionNotFound(position) => {
                write!(formatter, "new context function not found\n{}", position)
            }
            Self::VariantTypeInFfi(position) => {
                write!(
                    formatter,
                    "union and any type not supported in FFI\n{}",
                    position
                )
            }
        }
    }
}

impl Error for CompileError {}

impl From<mir::analysis::TypeCheckError> for CompileError {
    fn from(error: mir::analysis::TypeCheckError) -> Self {
        Self::MirTypeCheck(error)
    }
}

impl From<AnalysisError> for CompileError {
    fn from(error: AnalysisError) -> Self {
        Self::Analysis(error)
    }
}
