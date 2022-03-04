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
    DuplicateFunctionNames(Position, Position),
    DuplicateTypeNames(Position, Position),
    FunctionEqualOperation(Position),
    FunctionExpected(Position),
    InvalidRecordEqualOperation(Position),
    InvalidTryOperation(Position),
    MainFunctionNotFound(Position),
    MirTypeCheck(mir::analysis::TypeCheckError),
    UnusedErrorValue(Position),
    NewContextFunctionNotFound(Position),
    RecordFieldPrivate(Position),
    RecordFieldUnknown(Position),
    RecordExpected(Position),
    TryOperationInList(Position),
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
            Self::DuplicateFunctionNames(one, other) => {
                write!(formatter, "duplicate function names\n{}\n{}", one, other)
            }
            Self::DuplicateTypeNames(one, other) => {
                write!(formatter, "duplicate type names\n{}\n{}", one, other)
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
            Self::InvalidTryOperation(position) => {
                write!(
                    formatter,
                    "try operation cannot be used in function not returning error\n{}",
                    position
                )
            }
            Self::MainFunctionNotFound(position) => {
                write!(formatter, "main function not found\n{}", position)
            }
            Self::MirTypeCheck(error) => {
                write!(formatter, "failed to check types in MIR: {}", error)
            }
            Self::UnusedErrorValue(position) => {
                write!(formatter, "unused error value\n{}", position)
            }
            Self::NewContextFunctionNotFound(position) => {
                write!(formatter, "new context function not found\n{}", position)
            }
            Self::RecordFieldPrivate(position) => {
                write!(formatter, "private record field\n{}", position)
            }
            Self::RecordFieldUnknown(position) => {
                write!(formatter, "unknown record field\n{}", position)
            }
            Self::RecordExpected(position) => {
                write!(formatter, "record expected\n{}", position)
            }
            Self::TryOperationInList(position) => {
                write!(
                    formatter,
                    "try operation not allowed in list literal\n{}",
                    position
                )
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
