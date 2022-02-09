use hir::{analysis::types::TypeError, ir::*, types};
use position::Position;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    AnyEqualOperation(Position),
    AnyTypeBranch(Position),
    CompileConfigurationNotProvided,
    DuplicateFunctionNames(Position, Position),
    DuplicateTypeNames(Position, Position),
    FunctionEqualOperation(Position),
    FunctionExpected(Position),
    InvalidRecordEqualOperation(Position),
    InvalidTryOperation(Position),
    ListExpected(Position),
    MainFunctionNotFound(Position),
    MirTypeCheck(mir::analysis::TypeCheckError),
    MissingElseBlock(Position),
    NewContextFunctionNotFound(Position),
    RecordFieldPrivate(Position),
    RecordFieldUnknown(Position),
    RecordFieldMissing(Position),
    RecordExpected(Position),
    RecordNotFound(types::Record),
    SpawnOperationArguments(Position),
    TryOperationInList(Position),
    TypeAnalysis(TypeError),
    TypeNotFound(types::Reference),
    TypeNotInferred(Position),
    TypesNotComparable(Position),
    TypesNotMatched(Position, Position),
    UnionOrAnyTypeExpected(Position),
    UnionTypeExpected(Position),
    UnreachableCode(Position),
    VariableNotFound(Variable),
    WrongArgumentCount(Position),
}

impl Display for CompileError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::AnyEqualOperation(position) => {
                write!(
                    formatter,
                    "equal operator cannot be used with any type\n{}",
                    position
                )
            }
            Self::AnyTypeBranch(position) => {
                write!(
                    formatter,
                    "any type cannot be used for downcast\n{}",
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
            Self::ListExpected(position) => {
                write!(formatter, "list expected\n{}", position)
            }
            Self::MainFunctionNotFound(position) => {
                write!(formatter, "main function not found\n{}", position)
            }
            Self::MirTypeCheck(error) => {
                write!(formatter, "failed to check types in MIR: {}", error)
            }
            Self::MissingElseBlock(position) => {
                write!(
                    formatter,
                    "missing else block in if-type expression\n{}",
                    position
                )
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
            Self::RecordFieldMissing(position) => {
                write!(formatter, "missing record field\n{}", position)
            }
            Self::RecordExpected(position) => {
                write!(formatter, "record expected\n{}", position)
            }
            Self::RecordNotFound(record) => write!(
                formatter,
                "record type \"{}\" not found\n{}",
                record.name(),
                record.position()
            ),
            Self::SpawnOperationArguments(position) => {
                write!(
                    formatter,
                    "lambda expression in spawn operation cannot have any argument\n{}",
                    position
                )
            }
            Self::TryOperationInList(position) => {
                write!(
                    formatter,
                    "try operation not allowed in list literal\n{}",
                    position
                )
            }
            Self::TypeAnalysis(error) => write!(formatter, "{}", error),
            Self::TypeNotFound(reference) => write!(
                formatter,
                "type \"{}\" not found\n{}",
                reference.name(),
                reference.position()
            ),
            Self::TypeNotInferred(position) => {
                write!(formatter, "type not inferred\n{}", position)
            }
            Self::TypesNotComparable(position) => {
                write!(formatter, "types not comparable\n{}", position)
            }
            Self::TypesNotMatched(lhs_position, rhs_position) => write!(
                formatter,
                "types not matched\n{}\n{}",
                lhs_position, rhs_position
            ),
            Self::UnionOrAnyTypeExpected(position) => {
                write!(formatter, "union or any type expected\n{}", position)
            }
            Self::UnionTypeExpected(position) => {
                write!(formatter, "union type expected\n{}", position)
            }
            Self::UnreachableCode(position) => {
                write!(formatter, "unreachable code\n{}", position)
            }
            Self::VariableNotFound(variable) => write!(
                formatter,
                "variable \"{}\" not found\n{}",
                variable.name(),
                variable.position()
            ),
            Self::WrongArgumentCount(position) => {
                write!(
                    formatter,
                    "wrong number of arguments in function call\n{}",
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

impl From<TypeError> for CompileError {
    fn from(error: TypeError) -> Self {
        Self::TypeAnalysis(error)
    }
}
