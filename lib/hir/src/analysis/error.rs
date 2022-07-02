use crate::{ir::*, types::*};
use position::Position;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnalysisError {
    AnyTypeBranch(Position),
    CollectionExpected(Position),
    DuplicateFunctionNames(Position, Position),
    DuplicateTypeNames(Position, Position),
    ErrorTypeUndefined,
    FunctionExpected(Position),
    ImpossibleRecord(Position),
    InvalidTryOperation(Position),
    ListExpected(Position),
    MapExpected(Position),
    MissingElseBlock(Position),
    RecordExpected(Position),
    RecordFieldMissing(Position),
    RecordFieldPrivate(Position),
    RecordFieldUnknown(Position),
    RecordNotFound(Record),
    RecursiveTypeAlias(Position),
    SpawnOperationArguments(Position),
    TryOperationInList(Position),
    TypeNotFound(Reference),
    TypeNotInferred(Position),
    TypeNotComparable(Position),
    TypesNotMatched(Position, Position),
    UnionExpected(Position),
    UnknownRecordField(Position),
    UnreachableCode(Position),
    UnusedErrorValue(Position),
    VariableNotFound(Variable),
    VariantExpected(Position),
    WrongArgumentCount(Position),
}

impl Display for AnalysisError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::AnyTypeBranch(position) => {
                write!(
                    formatter,
                    "any type cannot be used for downcast\n{}",
                    position
                )
            }
            Self::CollectionExpected(position) => {
                write!(formatter, "list or map expected\n{}", position)
            }
            Self::DuplicateFunctionNames(one, other) => {
                write!(formatter, "duplicate function names\n{}\n{}", one, other)
            }
            Self::DuplicateTypeNames(one, other) => {
                write!(formatter, "duplicate type names\n{}\n{}", one, other)
            }
            Self::ErrorTypeUndefined => {
                write!(formatter, "error type undefined")
            }
            Self::FunctionExpected(position) => {
                write!(formatter, "function expected\n{}", position)
            }
            Self::ImpossibleRecord(position) => {
                write!(
                    formatter,
                    "record construction dependent on itself\n{}",
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
            Self::MapExpected(position) => {
                write!(formatter, "map expected\n{}", position)
            }
            Self::MissingElseBlock(position) => {
                write!(
                    formatter,
                    "missing else block in if-type expression\n{}",
                    position
                )
            }
            Self::RecordExpected(position) => {
                write!(formatter, "record expected\n{}", position)
            }
            Self::RecordFieldMissing(position) => {
                write!(formatter, "missing record field\n{}", position)
            }
            Self::RecordFieldPrivate(position) => {
                write!(formatter, "private record field\n{}", position)
            }
            Self::RecordFieldUnknown(position) => {
                write!(formatter, "unknown record field\n{}", position)
            }
            Self::RecordNotFound(record) => write!(
                formatter,
                "record type \"{}\" not found\n{}",
                record.name(),
                record.position()
            ),
            Self::RecursiveTypeAlias(position) => {
                write!(formatter, "recursive type alias\n{}", position)
            }
            Self::TryOperationInList(position) => {
                write!(
                    formatter,
                    "try operation not allowed in list literal\n{}",
                    position
                )
            }
            Self::SpawnOperationArguments(position) => {
                write!(
                    formatter,
                    "lambda expression in spawn operation cannot have any argument\n{}",
                    position
                )
            }
            Self::TypeNotComparable(position) => {
                write!(formatter, "types not comparable\n{}", position)
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
            Self::TypesNotMatched(lhs_position, rhs_position) => write!(
                formatter,
                "types not matched\n{}\n{}",
                lhs_position, rhs_position
            ),
            Self::UnionExpected(position) => {
                write!(formatter, "union type expected\n{}", position)
            }
            Self::UnknownRecordField(position) => {
                write!(formatter, "unknown record field\n{}", position)
            }
            Self::UnreachableCode(position) => {
                write!(formatter, "unreachable code\n{}", position)
            }
            Self::UnusedErrorValue(position) => {
                write!(formatter, "unused error value\n{}", position)
            }
            Self::VariableNotFound(variable) => write!(
                formatter,
                "variable \"{}\" not found\n{}",
                variable.name(),
                variable.position()
            ),
            Self::VariantExpected(position) => {
                write!(formatter, "union or any type expected\n{}", position)
            }
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

impl Error for AnalysisError {}
