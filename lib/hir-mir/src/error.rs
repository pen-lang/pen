use hir::{
    analysis::{type_formatter, AnalysisError},
    types::Type,
};
use position::Position;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    Analysis(AnalysisError),
    CompileConfigurationNotProvided,
    InvalidRecordEqualOperation(Position),
    InvalidVariantType(Type),
    MainFunctionNotFound(Position),
    MirTypeCheck(mir::analysis::type_check::TypeCheckError),
    MixedIterateesInListComprehension(Position),
    MultipleMapsInListComprehension(Position),
    NewContextFunctionNotFound(Position),
}

impl Display for CompileError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Analysis(error) => write!(formatter, "{}", error),
            Self::CompileConfigurationNotProvided => {
                write!(formatter, "compile configuration not provided")
            }
            Self::InvalidRecordEqualOperation(position) => {
                write!(
                    formatter,
                    "equal operator cannot be used with record type containing any or function types\n{}",
                    position
                )
            }
            Self::InvalidVariantType(type_) => {
                write!(
                    formatter,
                    "unsupported type information: {}\n{}",
                    type_formatter::format(type_),
                    type_.position()
                )
            }
            Self::MainFunctionNotFound(position) => {
                write!(formatter, "main function not found\n{}", position)
            }
            Self::MirTypeCheck(error) => {
                write!(formatter, "failed to check types in MIR: {}", error)
            }
            Self::MixedIterateesInListComprehension(position) => {
                write!(
                    formatter,
                    "mixed iteratee types in list comprehension\n{}",
                    position
                )
            }
            Self::MultipleMapsInListComprehension(position) => {
                write!(
                    formatter,
                    "multiple maps in list comprehension\n{}",
                    position
                )
            }
            Self::NewContextFunctionNotFound(position) => {
                write!(formatter, "new context function not found\n{}", position)
            }
        }
    }
}

impl Error for CompileError {}

impl From<mir::analysis::type_check::TypeCheckError> for CompileError {
    fn from(error: mir::analysis::type_check::TypeCheckError) -> Self {
        Self::MirTypeCheck(error)
    }
}

impl From<AnalysisError> for CompileError {
    fn from(error: AnalysisError) -> Self {
        Self::Analysis(error)
    }
}
