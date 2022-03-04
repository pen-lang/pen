use crate::{ir::*, types::*};
use position::Position;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, PartialEq)]
pub enum AnalysisError {
    AnyTypeBranch(Position),
    ErrorTypeUndefined,
    FunctionExpected(Position),
    ListExpected(Position),
    MissingElseBlock(Position),
    RecordExpected(Position),
    RecordFieldMissing(Position),
    RecordNotFound(Record),
    SpawnOperationArguments(Position),
    TypesNotComparable(Position),
    TypeNotFound(Reference),
    TypeNotInferred(Position),
    TypesNotMatched(Position, Position),
    UnionExpected(Position),
    UnknownRecordField(Position),
    UnreachableCode(Position),
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
            Self::ErrorTypeUndefined => {
                write!(formatter, "error type undefined")
            }
            Self::FunctionExpected(position) => {
                write!(formatter, "function expected\n{}", position)
            }
            Self::ListExpected(position) => {
                write!(formatter, "list expected\n{}", position)
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
            Self::TypesNotComparable(position) => {
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
